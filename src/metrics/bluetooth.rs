use std::fmt::Display;
use std::process::Command;
use std::time::Duration;

use crate::Metric;

#[derive(Debug, Default)]
pub struct BluetoothChargeMetric {}

impl BluetoothChargeMetric {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Metric for BluetoothChargeMetric {
    fn timeout(&self) -> std::time::Duration {
        Duration::from_secs(20)
    }
}

impl Display for BluetoothChargeMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: rewrite from shell api
        let out = Command::new("sh")
            .arg("-c")
            .arg("bluetoothctl info | grep 'Battery Percentage' | sed 's/.*(\\([^)]*\\)).*/\\1/g'")
            .output()
            .map_err(|_| std::fmt::Error)?;

        let percentage = std::str::from_utf8(&out.stdout).map_err(|_| std::fmt::Error)?;
        let percentage = percentage.trim();

        if !percentage.is_empty() {
            write!(f, "🎧⚡️ {percentage}%")?;
        }

        Ok(())
    }
}
