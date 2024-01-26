use rand::thread_rng;
use rand::Rng;
use std::ops::Range;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SleepDuration {
    Duration(Duration),
    MinMax(Duration, Duration),
}

impl SleepDuration {
    pub const fn from_duration(duration: Duration) -> Self {
        Self::Duration(duration)
    }

    pub const fn from_min_max_durations(min_duration: Duration, max_duration: Duration) -> Self {
        Self::MinMax(min_duration, max_duration)
    }

    pub const fn from_millis(millis: u64) -> Self {
        Self::Duration(Duration::from_millis(millis))
    }

    pub const fn from_min_max_millis(min_millis: u64, max_millis: u64) -> Self {
        Self::MinMax(
            Duration::from_millis(min_millis),
            Duration::from_millis(max_millis),
        )
    }

    pub fn into_duration(self) -> Duration {
        match self {
            Self::Duration(duration) => duration,
            Self::MinMax(min, max) => thread_rng().gen_range(min..max),
        }
    }

    pub async fn sleep(self) {
        sleep(self.into_duration()).await
    }
}

impl From<SleepDuration> for Duration {
    fn from(value: SleepDuration) -> Self {
        value.into_duration()
    }
}

impl From<Duration> for SleepDuration {
    fn from(duration: Duration) -> Self {
        Self::Duration(duration)
    }
}

impl From<Range<Duration>> for SleepDuration {
    fn from(min_max: Range<Duration>) -> Self {
        Self::MinMax(min_max.start, min_max.end)
    }
}

impl From<(Duration, Duration)> for SleepDuration {
    fn from((min, max): (Duration, Duration)) -> Self {
        Self::MinMax(min, max)
    }
}
