use crate::Metric;
use std::process::Command;
use std::time::Duration;

pub struct MemMetric {
}

impl MemMetric {
  #[allow(dead_code)]
  pub fn new() -> MemMetric {
    MemMetric { }
  }
}

impl Metric for MemMetric {
  fn get_timeout(&self) -> Duration {
    Duration::ZERO
  }
  fn update(&mut self) {}
  fn get_value(&self) -> String {
    // TODO: rewrite from shell api
    let out = Command::new("sh")
      .arg("-c")
      .arg("free -h | awk '/Mem/ {printf \"%s/%s\n\", $3, $2}'")
      .output()
      .expect("Failed to get xkb layout")
      .stdout;
    format!("📝 {}", String::from_utf8_lossy(&out))
  }
}
