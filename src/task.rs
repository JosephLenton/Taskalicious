use ::std::future::Future;

pub trait Task {
    type Output;
    fn call(&mut self) -> impl Future<Output = Self::Output>;
}
