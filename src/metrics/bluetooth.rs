use crate::Metric;
use std::ops::Not;
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
    fn get_value(&self) -> Option<String> {
        // TODO: rewrite from shell api
        let out = Command::new("sh")
            .arg("-c")
            .arg("bluetoothctl info | grep 'Battery Percentage' | sed 's/.*(\\([^)]*\\)).*/\\1/g'")
            .output()
            .ok()?;

        let percentage = String::from_utf8_lossy(&out.stdout).to_string();

        percentage
            .is_empty()
            .not()
            .then(|| format!("🎧⚡️ {}%", percentage))
    }
}
