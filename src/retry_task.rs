use tokio::time::sleep;

use crate::Retry;
use crate::Task;

#[derive(Copy, Clone, Debug)]
pub struct RetryTask<T>
where
    T: Task,
{
    task: T,
    retries: Retry,
}

impl<T> RetryTask<T>
where
    T: Task,
{
    pub fn new(task: T, retries: Retry) -> Self {
        Self { task, retries }
    }
}

impl<T, O, E> Task for RetryTask<T>
where
    T: Task<Output = Result<O, E>>,
{
    type Output = T::Output;

    async fn call(&mut self) -> Result<O, E>
    where
        T: Task<Output = Result<O, E>>,
    {
        let num_retries = self.retries.num_retries;
        let sleep_time = self.retries.sleep_time;

        let mut last_error: Option<E> = None;

        for retry in 0..num_retries {
            let result = self.task.call().await;

            match result {
                Err(err) => {
                    last_error = Some(err);

                    if retry < num_retries {
                        sleep(sleep_time.into_duration()).await;
                        continue;
                    }
                }
                Ok(return_value) => return Ok(return_value),
            }
        }

        match last_error {
            Some(error) => Err(error),
            None => {
                panic!("Ran out of retries and error was not caught (this should not be possible)")
            }
        }
    }
}

#[cfg(test)]
mod test_run {
    use super::*;

    use crate::FnTask;
    use crate::TaskExt;
    use anyhow::anyhow;
    use anyhow::bail;
    use anyhow::Result;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    #[tokio::test]
    async fn it_should_call_once_and_return_if_all_ok() {
        let num_calls = AtomicU32::new(0);
        let task = FnTask::new(|| async {
            num_calls.fetch_add(1, Ordering::Acquire);
            Ok(()) as Result<()>
        });

        let mut retries = RetryTask::new(task, Retry::new());
        let result = retries.call().await;

        assert!(result.is_ok());
        assert_eq!(num_calls.into_inner(), 1);
    }

    #[tokio::test]
    async fn it_should_call_multiple_times_until_ok_and_no_more() {
        let num_calls = AtomicU32::new(0);

        let retries = Retry::new()
            .sleep_time(Duration::from_millis(0))
            .retries(10);

        let task = FnTask::new(|| async {
            let current_val = num_calls.fetch_add(1, Ordering::Acquire) + 1;
            if current_val < 5 {
                bail!("not enough calls");
            }

            Ok(()) as Result<()>
        });
        let result = task.with_retry(retries).call().await;

        assert!(result.is_ok());
        assert_eq!(num_calls.into_inner(), 5);
    }

    #[tokio::test]
    async fn it_should_wait_for_sleep_time() {
        let num_calls = AtomicU32::new(0);

        let retries = Retry::new()
            .sleep_time(Duration::from_millis(100))
            .retries(10);

        let task = FnTask::new(|| async {
            let current_val = num_calls.fetch_add(1, Ordering::Acquire) + 1;
            if current_val < 4 {
                bail!("not enough calls");
            }

            Ok(()) as Result<()>
        });

        let start = std::time::Instant::now();
        let result = RetryTask::new(task, retries).call().await;
        let end = std::time::Instant::now();
        let time_taken = end - start;

        assert!(result.is_ok());
        assert!(time_taken >= Duration::from_millis(300));
    }

    #[tokio::test]
    async fn it_should_return_error_if_out_of_retries() {
        let num_calls = AtomicU32::new(0);

        let retries = Retry::new()
            .sleep_time(Duration::from_millis(0))
            .retries(10);

        let task = FnTask::new(|| async {
            num_calls.fetch_add(1, Ordering::Acquire);

            Err(anyhow!("always fail")) as Result<()>
        });

        let result = task.with_retry(retries).call().await;

        assert!(result.is_err());
        assert_eq!(num_calls.into_inner(), 10);
    }
}
