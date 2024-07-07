use std::fmt::Display;
use std::process::Command;

pub struct BluetoothChargeMetric {}

#[allow(clippy::new_without_default)]
impl BluetoothChargeMetric {
    #[must_use]
    pub fn new() -> Self {
        Self {}
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

        if !percentage.is_empty() {
            write!(f, "🎧⚡️ {percentage}%")?;
        }

        Ok(())
    }
}
