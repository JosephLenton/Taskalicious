use anyhow::anyhow;
use anyhow::Error as AnyhowError;
use std::future::Future;
use tokio::time::sleep;

use crate::SleepTime;

const DEFAULT_NUM_RETRIES: u32 = 3;
const DEFAULT_SLEEP_TIME: SleepTime = SleepTime::from_millis(10_000);

#[derive(Copy, Clone, Debug)]
pub struct RetryTask {
    num_retries: u32,
    sleep_time: SleepTime,
}

impl RetryTask {
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

    pub async fn run<'a, V, F, R, E>(self, fun: F) -> Result<V, AnyhowError>
    where
        F: Fn() -> R + 'a,
        R: Future<Output = Result<V, E>> + 'a,
        E: Into<AnyhowError>,
    {
        let num_retries = self.num_retries;
        let sleep_time = self.sleep_time;

        let mut last_error: Option<AnyhowError> = None;

        for retry in 0..num_retries {
            let result = fun().await;

            match result {
                Err(err) => {
                    let anyhow_error: AnyhowError = err.into();
                    eprintln!("{:?}", anyhow_error);
                    last_error = Some(anyhow_error);

                    if retry < num_retries {
                        sleep(sleep_time.into_duration()).await;
                        continue;
                    }
                }
                Ok(return_value) => return Ok(return_value),
            }
        }

        let error = last_error.unwrap_or_else(|| {
            anyhow!("Ran out of retries and error was not caught (this should not be possible)")
        });
        Err(error)
    }
}

#[cfg(test)]
mod test_run {
    use super::*;

    use anyhow::bail;
    use anyhow::Result;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    #[tokio::test]
    async fn it_should_call_once_and_return_if_all_ok() {
        let num_calls = AtomicU32::new(0);

        let retries = RetryTask::new().retries(10);

        let result = retries
            .run(|| async {
                num_calls.fetch_add(1, Ordering::Acquire);
                Ok(()) as Result<()>
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(num_calls.into_inner(), 1);
    }

    #[tokio::test]
    async fn it_should_call_multiple_times_until_ok_and_no_more() {
        let num_calls = AtomicU32::new(0);

        let retries = RetryTask::new()
            .sleep_time(Duration::from_millis(0))
            .retries(10);

        let result = retries
            .run(|| async {
                let current_val = num_calls.fetch_add(1, Ordering::Acquire) + 1;
                if current_val < 5 {
                    bail!("not enough calls");
                }

                Ok(()) as Result<()>
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(num_calls.into_inner(), 5);
    }

    #[tokio::test]
    async fn it_should_wait_for_sleep_time() {
        let num_calls = AtomicU32::new(0);

        let retries = RetryTask::new()
            .sleep_time(Duration::from_millis(100))
            .retries(10);

        let start = std::time::Instant::now();
        let result = retries
            .run(|| async {
                let current_val = num_calls.fetch_add(1, Ordering::Acquire) + 1;
                if current_val < 4 {
                    bail!("not enough calls");
                }

                Ok(()) as Result<()>
            })
            .await;
        let end = std::time::Instant::now();
        let time_taken = end - start;

        assert!(result.is_ok());
        assert!(time_taken >= Duration::from_millis(300));
    }

    #[tokio::test]
    async fn it_should_return_error_if_out_of_retries() {
        let num_calls = AtomicU32::new(0);

        let retries = RetryTask::new()
            .sleep_time(Duration::from_millis(0))
            .retries(10);

        let result = retries
            .run(|| async {
                num_calls.fetch_add(1, Ordering::Acquire);

                Err(anyhow!("always fail")) as Result<()>
            })
            .await;

        assert!(result.is_err());
        assert_eq!(num_calls.into_inner(), 10);
    }
}
