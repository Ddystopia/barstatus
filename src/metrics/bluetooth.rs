use crate::Metric;
use std::process::Command;
use std::time::Duration;

pub struct BluetoothChargeMetric {}

#[allow(clippy::new_without_default)]
impl BluetoothChargeMetric {
  pub fn new() -> BluetoothChargeMetric {
    BluetoothChargeMetric {}
  }
}

impl Metric for BluetoothChargeMetric {
  fn get_timeout(&self) -> Duration {
    Duration::ZERO
  }
  fn update(&mut self) {}
  fn get_value(&self) -> String {
    // TODO: rewrite from shell api
    let out = Command::new("sh")
      .arg("-c")
      .arg("bluetoothctl info | grep 'Battery Percentage' | sed 's/.*(\\([^)]*\\)).*/\\1/g'")
      .output();

    let Ok(out) = out else {
      return String::new();
    };

    let percentage = String::from_utf8_lossy(&out.stdout).to_string();
    if !percentage.is_empty() {
      return format!("🎧⚡️ {}%", percentage);
    };
    percentage
  }
}
