use std::{
    fmt::{Display, Formatter},
    time::Duration,
};

use crate::{read_line::read_line_from_path, Metric};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BatteryMetric {
    threshold: u8,
}

impl BatteryMetric {
    #[must_use]
    pub fn new(threshold: u8) -> Self {
        Self { threshold }
    }

    fn emoji(&self) -> &'static str {
        match read_line_from_path::<24>("/sys/class/power_supply/BAT0/status") {
            Ok(status) if status.trim() == "Charging" => "🔌🔼",
            Ok(status) if status.trim() == "Discharging" => "🔋🔽",
            _ => "🔋",
        }
    }

    fn percentage(&self) -> Option<u8> {
        let p = read_line_from_path::<24>("/sys/class/power_supply/BAT0/capacity").ok()?;
        p.trim().parse::<u8>().ok()
    }
}

impl Metric for BatteryMetric {
    fn timeout(&self) -> Duration {
        Duration::ZERO
    }
}

impl Display for BatteryMetric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let emoji = self.emoji();
        let percentage = self.percentage().ok_or(std::fmt::Error)?;

        if percentage < self.threshold {
            write!(f, "{emoji} {percentage}%")?;
        }

        Ok(())
    }
}
