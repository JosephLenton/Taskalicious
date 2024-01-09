use crate::Task;
use ::std::future::Future;

pub struct FnTask<T, F, O>
where
    T: FnMut() -> F,
    F: Future<Output = O>,
{
    fun: T,
}

impl<T, F, O> FnTask<T, F, O>
where
    T: FnMut() -> F,
    F: Future<Output = O>,
{
    pub fn new(fun: T) -> Self {
        Self { fun }
    }
}

impl<T, F, O> Task for FnTask<T, F, O>
where
    T: FnMut() -> F,
    F: Future<Output = O>,
{
    type Output = O;

    fn call(&mut self) -> impl Future<Output = Self::Output> {
        (self.fun)()
    }
}

impl<T, F, O> From<T> for FnTask<T, F, O>
where
    T: FnMut() -> F,
    F: Future<Output = O>,
{
    fn from(fun: T) -> Self {
        FnTask::new(fun)
    }
}
