use std::time::{Duration, SystemTime, SystemTimeError};
pub mod emojis;
pub mod metrics;

pub trait Metric {
    fn update(&mut self) {}
    fn get_timeout(&self) -> Duration;
    fn get_value(&mut self) -> Option<String>;
}

fn duration_since(timestamp: SystemTime) -> Result<Duration, SystemTimeError> {
    SystemTime::now().duration_since(timestamp)
}
