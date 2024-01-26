use crate::FnTask;
use crate::RetryTask;
use crate::SleepDuration;
use crate::Task;
use ::std::future::Future;

const DEFAULT_NUM_RETRIES: u32 = 3;
const DEFAULT_SLEEP_DURATION: SleepDuration = SleepDuration::from_millis(10_000);

#[derive(Copy, Clone, Debug)]
pub struct Retry {
    num_retries: u32,
    sleep_duration: SleepDuration,
}

impl Retry {
    pub fn new() -> Self {
        Self {
            num_retries: DEFAULT_NUM_RETRIES,
            sleep_duration: DEFAULT_SLEEP_DURATION,
        }
    }

    pub fn retries(self) -> u32 {
        self.num_retries
    }

    pub fn set_retries(self, num_retries: u32) -> Self {
        Self {
            num_retries,
            ..self
        }
    }

    pub fn sleep_duration(self) -> SleepDuration {
        self.sleep_duration
    }

    pub fn set_sleep_duration<S>(self, sleep_duration: S) -> Self
    where
        S: Into<SleepDuration>,
    {
        Self {
            sleep_duration: sleep_duration.into(),
            ..self
        }
    }

    pub async fn run_fn<'a, T, F, O, E>(self, fn_task: T) -> Result<O, E>
    where
        T: FnMut() -> F,
        F: Future<Output = Result<O, E>>,
    {
        let task = FnTask::new(fn_task);
        self.run(task).await
    }

    pub async fn run<'a, T, O, E>(self, task: T) -> Result<O, E>
    where
        T: Task<Output = Result<O, E>>,
    {
        self.build_task(task).call().await
    }

    pub fn build_task<'a, T, O, E>(self, task: T) -> RetryTask<T>
    where
        T: Task<Output = Result<O, E>>,
    {
        RetryTask::new(task.into(), self)
    }
}
