use crate::Metric;
use std::process::Command;
use std::time::Duration;

pub struct MemMetric {}

#[allow(clippy::new_without_default)]
impl MemMetric {
  #[allow(dead_code)]
  pub fn new() -> MemMetric {
    MemMetric {}
  }
}

impl Metric for MemMetric {
  fn get_timeout(&self) -> Duration {
    Duration::ZERO
  }
  fn update(&mut self) {}
  fn get_value(&self) -> String {
    // TODO: rewrite from shell api
    return match Command::new("sh")
      .arg("-c")
      .arg("free -h | awk '/Mem/ {printf \"%s/%s\n\", $3, $2}'")
      .output()
    {
      Err(_) => String::new(),
      Ok(out) => format!("📝 {}", String::from_utf8_lossy(&out.stdout)),
    };
  }
}
