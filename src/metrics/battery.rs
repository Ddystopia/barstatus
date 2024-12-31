use std::{
    cell::Cell,
    fmt::{Display, Formatter},
    time::Duration,
};

use crate::{read_line::read_line_from_path, Metric};

#[derive(Debug, Clone)]
pub struct BatteryMetric {
    threshold: u8,
    display: Cell<DisplayBattery>,
}

#[derive(Default, Debug, Clone, Copy)]
struct DisplayBattery(&'static str, Option<u8>, u8);

impl BatteryMetric {
    #[must_use]
    pub fn new(threshold: u8) -> Self {
        Self {
            threshold,
            display: Default::default(),
        }
    }

    async fn emoji(&self) -> &'static str {
        match read_line_from_path::<24>("/sys/class/power_supply/BAT0/status").await {
            Ok(status) if status.trim() == "Charging" => "🔌🔼",
            Ok(status) if status.trim() == "Discharging" => "🔋🔽",
            _ => "🔋",
        }
    }

    async fn percentage(&self) -> Option<u8> {
        let percentage = read_line_from_path::<24>("/sys/class/power_supply/BAT0/capacity");

        percentage.await.ok()?.trim().parse::<u8>().ok()
    }
}

impl Metric for BatteryMetric {
    fn name(&self) -> &'static str {
        "Battery"
    }

    fn display(&self) -> impl Display {
        self.display.get()
    }

    fn start(&self) -> impl std::future::Future<Output = !> + '_ {
        async {
            loop {
                let emoji = self.emoji().await;
                let percentage = self.percentage().await;

                self.display
                    .set(DisplayBattery(emoji, percentage, self.threshold));

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

impl Display for DisplayBattery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self(emoji, percentage, threeshold) = self;
        if let Some(percentage) = percentage {
            if percentage < threeshold {
                write!(f, "{emoji} {percentage}%")?;
            }
        }

        Ok(())
    }
}
