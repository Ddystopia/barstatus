use crate::Metric;
use std::process::Command;
use std::time::Duration;

pub struct XkbLayoutMetric {
  timeout: Duration,
}

impl XkbLayoutMetric {
  pub fn new(timeout: Duration) -> XkbLayoutMetric {
    XkbLayoutMetric { timeout }
  }
}

impl Metric for XkbLayoutMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn update(&mut self) -> () {}
  fn get_value(&self) -> String {
    // TODO: rewrite from shell api
    let out = Command::new("sh")
      .arg("-c")
      .arg("xkb-switch")
      .output()
      .expect("Failed to get xkb layout")
      .stdout;
    let mut loc = String::from_utf8_lossy(&out).to_string();
    loc.pop();
    format!("🌍 {}", loc)
  }
}
