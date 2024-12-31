use std::cell::Cell;
use std::fmt::Display;
use tokio::process::Command;

use crate::{CommonError, Metric};

#[derive(Debug, Default)]
pub struct BluetoothChargeMetric(Cell<Option<u8>>);

impl Metric for BluetoothChargeMetric {
    fn name(&self) -> &'static str {
        "Bluetooth Charge"
    }

    fn display(&self) -> impl Display {
        self
    }

    async fn update(&self) -> Result<(), CommonError> {
        let cmd = "bluetoothctl info | grep 'Battery Percentage' | sed 's/.*(\\([^)]*\\)).*/\\1/g'";
        let result: Result<(), _> = try {
            // TODO: rewrite from shell api
            let out = Command::new("sh").arg("-c").arg(cmd).output().await?;
            let percentage = std::str::from_utf8(&out.stdout)?;
            let percentage = percentage.trim().parse::<u8>()?;
            self.0.set(Some(percentage));
        };

        if result.is_err() {
            self.0.set(None);
        }

        result
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
