use crate::RetryTask;
use crate::SleepTime;
use crate::Task;

const DEFAULT_NUM_RETRIES: u32 = 3;
const DEFAULT_SLEEP_TIME: SleepTime = SleepTime::from_millis(10_000);

#[derive(Copy, Clone, Debug)]
pub struct Retry {
    pub num_retries: u32,
    pub sleep_time: SleepTime,
}

impl Retry {
    pub fn new() -> Self {
        Self {
            num_retries: DEFAULT_NUM_RETRIES,
            sleep_time: DEFAULT_SLEEP_TIME,
        }
    }

    pub fn retries(self, num_retries: u32) -> Self {
        Self {
            num_retries,
            ..self
        }
    }

    pub fn sleep_time<S>(self, sleep_time: S) -> Self
    where
        S: Into<SleepTime>,
    {
        Self {
            sleep_time: sleep_time.into(),
            ..self
        }
    }

    pub async fn build_task<'a, T, O, E>(self, task: T) -> RetryTask<T>
    where
        T: Task<Output = Result<O, E>>,
    {
        RetryTask::new(task, self)
    }
}
