use std::cell::Cell;
use std::fmt::Display;
use std::process::Command;
use std::time::Duration;

use crate::Metric;

#[derive(Debug, Default)]
pub struct BluetoothChargeMetric(Cell<Option<u8>>);

impl BluetoothChargeMetric {
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }
}

impl Metric for BluetoothChargeMetric {
    fn name(&self) -> &'static str {
        "Bluetooth Charge"
    }

    fn display(&self) -> impl Display {
        self
    }

    fn start(&self) -> impl std::future::Future<Output = !> + '_ {
        async {
            loop {
                let cmd = "bluetoothctl info | grep 'Battery Percentage' | sed 's/.*(\\([^)]*\\)).*/\\1/g'";
                let percentage = try {
                    // TODO: rewrite from shell api
                    let out = Command::new("sh").arg("-c").arg(cmd).output().ok()?;
                    let percentage = std::str::from_utf8(&out.stdout).ok()?;
                    percentage.trim().parse::<u8>().ok()?
                };

                self.0.set(percentage);

                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

impl Display for BluetoothChargeMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let percentage = self.0.get();

        if let Some(percentage) = percentage {
            write!(f, "🎧⚡️ {percentage}%")?;
        }

        Ok(())
    }
}
