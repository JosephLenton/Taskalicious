use anyhow::anyhow;
use anyhow::Error as AnyhowError;
use anyhow::Result;
use std::future::Future;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct SuccessTrackingTask {
    is_alive: Arc<AtomicBool>,
    is_debug_print: bool,
}

impl SuccessTrackingTask {
    pub fn new() -> Self {
        Self {
            is_alive: Arc::new(AtomicBool::new(true)),
            is_debug_print: false,
        }
    }

    pub fn set_debug_print_err(&mut self) {
        self.is_debug_print = true;
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::Acquire)
    }

    pub fn abort(&self) {
        self.is_alive.store(false, Ordering::Release);
    }

    pub async fn run<F, R, E>(&self, task: F) -> Result<R>
    where
        F: Future<Output = Result<R, E>>,
        E: Into<AnyhowError>,
    {
        if !self.is_alive() {
            return Err(anyhow!("calling run on a task that has already ended"));
        }

        let result = task.await.map_err(Into::into);
        if let Err(err) = &result {
            if self.is_debug_print {
                eprintln!("Success Tracking Error failed (run) | {:?}", err)
            }

            self.abort();
        }

        result
    }

    pub async fn while_alive<T, E>(&self, mut task: T) -> Result<()>
    where
        T: crate::task::Task<Output = Result<(), E>>,
        E: Into<AnyhowError>,
    {
        if !self.is_alive() {
            return Err(anyhow!("calling run on a task that has already ended"));
        }

        while self.is_alive() {
            let result = task.call().await.map_err(Into::into);

            if let Err(err) = result {
                if self.is_debug_print {
                    eprintln!("Success Tracking Error failed (while_alive) | {:?}", err)
                }

                self.abort();
                return Err(err.into());
            }
        }

        // This will be returned if something else killed this.
        Ok(())
    }

    // Runs the given task in a new blocking thread, on it's own.
    //
    // It will spin there for as long as this is alive.
    pub fn spawn_while_alive<T, E>(&self, task: T) -> JoinHandle<Result<()>>
    where
        T: crate::task::Task<Output = Result<(), E>> + Send + 'static,
        E: Into<AnyhowError> + Send,
    {
        let clone = self.clone();
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async move { clone.while_alive(task).await })
        })
    }
}

impl Drop for SuccessTrackingTask {
    fn drop(&mut self) {
        self.abort()
    }
}

#[cfg(test)]
mod test_is_alive {
    use super::*;

    #[tokio::test]
    async fn it_should_be_alive_immediately() {
        let task = SuccessTrackingTask::new();

        assert_eq!(task.is_alive(), true);
    }

    #[tokio::test]
    async fn it_should_be_alive_after_ok_run() {
        let task = SuccessTrackingTask::new();
        let _ = task.run(async { Ok(()) as Result<()> }).await;

        assert_eq!(task.is_alive(), true);
    }

    #[tokio::test]
    async fn it_should_not_be_alive_after_err_run() {
        let task = SuccessTrackingTask::new();
        let _ = task.run(async { Err(anyhow!("fail")) as Result<()> }).await;

        assert_eq!(task.is_alive(), false);
    }

    #[tokio::test]
    async fn it_should_not_be_alive_after_calling_abort() {
        let task = SuccessTrackingTask::new();
        task.abort();

        assert_eq!(task.is_alive(), false);
    }
}

#[cfg(test)]
mod test_run {
    use super::*;

    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn it_should_run_immediately() {
        let num_calls = Arc::new(AtomicU32::new(0));
        let task = SuccessTrackingTask::new();

        let result = task
            .run(async {
                num_calls.fetch_add(1, Ordering::Acquire);

                Ok(()) as Result<()>
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(num_calls.load(Ordering::Acquire), 1);
    }

    #[tokio::test]
    async fn it_should_not_run_if_aborted() {
        let num_calls = Arc::new(AtomicU32::new(0));
        let task = SuccessTrackingTask::new();

        task.abort();
        let result = task
            .run(async {
                num_calls.fetch_add(1, Ordering::Acquire);

                Ok(()) as Result<()>
            })
            .await;

        assert!(result.is_err());
        assert_eq!(num_calls.load(Ordering::Acquire), 0);
    }

    #[tokio::test]
    async fn it_should_abort_clones() {
        let num_calls = Arc::new(AtomicU32::new(0));
        let task = SuccessTrackingTask::new();
        let clone = task.clone();

        task.abort();
        let result = clone
            .run(async {
                num_calls.fetch_add(1, Ordering::Acquire);

                Ok(()) as Result<()>
            })
            .await;

        assert!(result.is_err());
        assert_eq!(num_calls.load(Ordering::Acquire), 0);
    }

    #[tokio::test]
    async fn it_can_be_used_in_loop() {
        let num_calls = Arc::new(AtomicU32::new(0));
        let task = SuccessTrackingTask::new();

        while task.is_alive() {
            let _ = task
                .run(async {
                    let current_num = num_calls.fetch_add(1, Ordering::Acquire) + 1;

                    if current_num >= 3 {
                        return Err(anyhow!("Quit after 3 runs"));
                    }

                    Ok(()) as Result<()>
                })
                .await;
        }

        assert_eq!(num_calls.load(Ordering::Acquire), 3);
    }
}

#[cfg(test)]
mod test_while_alive {
    use super::*;

    use crate::FnTask;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn it_should_abort_clones() {
        let num_calls = Arc::new(AtomicU32::new(0));
        let task = SuccessTrackingTask::new();
        let clone = task.clone();

        task.abort();
        let result = clone
            .while_alive(FnTask::new(|| async {
                num_calls.fetch_add(1, Ordering::Acquire);

                Ok(()) as Result<()>
            }))
            .await;

        assert!(result.is_err());
        assert_eq!(num_calls.load(Ordering::Acquire), 0);
    }

    #[tokio::test]
    async fn it_runs_loop_whilst_alive() {
        let num_calls = Arc::new(AtomicU32::new(0));
        let task = SuccessTrackingTask::new();

        let result = task
            .while_alive(FnTask::new(|| async {
                let current_num = num_calls.fetch_add(1, Ordering::Acquire) + 1;
                if current_num >= 3 {
                    return Err(anyhow!("Quit after 3 runs"));
                }

                Ok(()) as Result<()>
            }))
            .await;

        assert_eq!(num_calls.load(Ordering::Acquire), 3);
        assert!(result.is_err());
    }
}
