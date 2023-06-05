use crate::Metric;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::time::Duration;

fn read_line_from_path(path: &str) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut result = String::new();
    buf_reader.read_line(&mut result).map(|_| result)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BatteryMetric {
    threshold: u8,
}

impl BatteryMetric {
    pub fn new(threshold: u8) -> BatteryMetric {
        BatteryMetric { threshold }
    }
}

impl Metric for BatteryMetric {
    fn get_timeout(&self) -> Duration {
        Duration::ZERO
    }

    fn get_value(&mut self) -> Option<String> {
        let percentage = read_line_from_path("/sys/class/power_supply/BAT0/capacity").ok()?;

        let emoji = match read_line_from_path("/sys/class/power_supply/BAT0/status") {
            Ok(status) if status.trim() == "Charging" => "🔌🔼",
            Ok(status) if status.trim() == "Discharging" => "🔋🔽",
            _ => "🔋",
        };

        let percentage = percentage.trim().parse::<u8>().ok()?;
        let less_than_threshold = percentage < self.threshold;

        less_than_threshold.then(|| format!("{} {}%", emoji, percentage))
    }
}
