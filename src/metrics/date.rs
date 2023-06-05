use crate::Metric;
use chrono::offset::Local;
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DateMetric {}

impl DateMetric {
    pub fn new() -> DateMetric {
        DateMetric {}
    }
}

impl Metric for DateMetric {
    fn get_timeout(&self) -> Duration {
        Duration::ZERO
    }
    fn get_value(&mut self) -> Option<String> {
        Some(
            Local::now()
                .naive_local()
                .format("%a, %b %d %X")
                .to_string(),
        )
    }
}
