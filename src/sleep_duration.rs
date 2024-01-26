use rand::thread_rng;
use rand::Rng;
use std::ops::Range;
use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SleepDuration {
    Duration(Duration),
    MinMax(Duration, Duration),
}

impl SleepDuration {
    pub const fn from_millis(millis: u64) -> Self {
        Self::Duration(Duration::from_millis(millis))
    }

    pub fn into_duration(self) -> Duration {
        match self {
            Self::Duration(duration) => duration,
            Self::MinMax(min, max) => thread_rng().gen_range(min..max),
        }
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
