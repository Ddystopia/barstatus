use crate::Metric;
use chrono::offset::Local;
use std::{fmt::Formatter, fmt::Display, time::Duration};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DateMetric {}

impl DateMetric {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Metric for DateMetric {
    fn timeout(&self) -> Duration {
        Duration::ZERO
    }
}

impl Display for DateMetric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fmt = Local::now().naive_local().format("%a, %b %d %X");
        write!(f, "{fmt}")
    }
}
