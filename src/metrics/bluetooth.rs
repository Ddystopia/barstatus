use crate::Metric;
use std::process::Command;
use std::time::Duration;

pub struct BluetoothChargeMetric {}

impl BluetoothChargeMetric {
  pub fn new() -> BluetoothChargeMetric {
    BluetoothChargeMetric {}
  }
}

impl Metric for BluetoothChargeMetric {
  fn get_timeout(&self) -> Duration {
    Duration::ZERO
  }
  fn update(&mut self) -> () {}
  fn get_value(&self) -> String {
    // TODO: rewrite from shell api
    let out = Command::new("sh")
      .arg("-c")
      .arg("bluetoothctl info | grep 'Battery Percentage' | sed 's/.*(\\([^)]*\\)).*/\\1/g'")
      .output();

    if let Err(_) = out {
      return String::new();
    }

    let out = out.unwrap().stdout;

    let percentage = String::from_utf8_lossy(&out).to_string();
    if percentage.len() > 0 {
      return format!("🎧⚡️ {}%", percentage);
    };
    return percentage;
  }
}
