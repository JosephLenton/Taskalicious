use tokio::task::JoinHandle;

use crate::Task;

pub trait TaskSpawnExt: Task + Send + Sized {
    fn spawn_blocking(self) -> JoinHandle<<Self as Task>::Output>;
}

impl<T, O, E> TaskSpawnExt for T
where
    T: Task<Output = Result<O, E>> + Send + 'static,
    O: Send + 'static,
    E: Send + 'static,
{
    fn spawn_blocking(mut self) -> JoinHandle<T::Output> {
        tokio::task::spawn_blocking(move || {
            let future = self.call();
            tokio::runtime::Handle::current().block_on(future)
        })
    }
}
