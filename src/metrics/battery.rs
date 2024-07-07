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

    fn get_value(&self) -> Option<String> {
        let percentage = read_line_from_path("/sys/class/power_supply/BAT0/capacity").ok()?;

        let emoji = match read_line_from_path("/sys/class/power_supply/BAT0/status") {
            Ok(status) if status.trim() == "Charging" => "🔌🔼",
            Ok(status) if status.trim() == "Discharging" => "🔋🔽",
            _ => "🔋",
        };

        let percentage = percentage.trim().parse::<u8>().ok()?;
        let less_than_threshold = percentage < self.threshold;

        less_than_threshold.then(|| format!("{emoji} {percentage}%"))
    }
}
