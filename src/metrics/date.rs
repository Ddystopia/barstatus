use crate::Metric;
use chrono::offset::Local;
use std::time::Duration;

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
  fn update(&mut self) -> () {}
  fn get_value(&self) -> String {
    Local::now()
      .naive_local()
      .format("%a, %b %d %X")
      .to_string()
  }
}
