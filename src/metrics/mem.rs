use crate::Metric;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
  fn get_value(&mut self) -> String {
    // TODO: rewrite from shell api
    return match Command::new("sh")
      .arg("-c")
      .arg("free -h | awk '/Mem/ {printf \"%s/%s\n\", $3, $2}'")
      .output()
    {
      Ok(out) => format!("📝 {}", String::from_utf8_lossy(&out.stdout)),
      Err(_) => String::new(),
    };
  }
}
