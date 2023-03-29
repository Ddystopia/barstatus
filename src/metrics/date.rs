use crate::Metric;
use chrono::offset::Local;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DateMetric {}

#[allow(clippy::new_without_default)]
impl DateMetric {
  pub fn new() -> DateMetric {
    DateMetric {}
  }
}

impl Metric for DateMetric {
  fn get_timeout(&self) -> Duration {
    Duration::ZERO
  }
  fn update(&mut self) {}
  fn get_value(&mut self) -> String {
    Local::now()
      .naive_local()
      .format("%a, %b %d %X")
      .to_string()
  }
}
