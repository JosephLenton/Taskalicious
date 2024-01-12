use crate::Retry;
use crate::RetryTask;
use crate::Task;

pub trait TaskExt: Task + Sized {
    fn with_retry(self, retries: Retry) -> RetryTask<Self>;
}

impl<T, O, E> TaskExt for T
where
    T: Task<Output = Result<O, E>>,
{
    fn with_retry(self, retries: Retry) -> RetryTask<Self> {
        RetryTask::new(self, retries)
    }
}
