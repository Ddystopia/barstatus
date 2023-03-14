#![feature(try_blocks)]
use std::time::{Duration, SystemTime, SystemTimeError};
// pub mod emojis;
pub mod metrics;

pub trait Metric {
  fn update(&mut self);
  fn get_timeout(&self) -> Duration;
  fn get_value(&self) -> String;
}

fn duration_since(timestamp: SystemTime) -> Result<Duration, SystemTimeError> {
  SystemTime::now().duration_since(timestamp)         
}
