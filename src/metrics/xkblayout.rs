use crate::Metric;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
  fn update(&mut self) {}
  fn get_value(&mut self) -> String {
    // TODO: rewrite from shell api
    return match Command::new("sh").arg("-c").arg("xkb-switch").output() {
      Err(_) => String::new(),
      Ok(out) => {
        let mut loc = String::from_utf8_lossy(&out.stdout).to_string();
        loc.pop();
        format!("🌍 {}", loc)
      }
    };
  }
}
