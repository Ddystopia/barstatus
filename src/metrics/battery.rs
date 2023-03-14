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
  fn update(&mut self) {}
  fn get_value(&self) -> String {
    // TODO: rewrite from shell api
    let percentage = match read_line_from_path("/sys/class/power_supply/BAT0/capacity") {
      Ok(percentage) => percentage,
      Err(_) => return String::new(),
    };
    let emoji = match read_line_from_path("/sys/class/power_supply/BAT0/status") {
      Ok(status) if status.trim() == "Charging" => "🔌🔼",
      Ok(status) if status.trim() == "Discharging" => "🔋🔽",
      _ => "🔋",
    };
    return match percentage.trim().parse::<u8>() {
      Ok(percentage) if percentage < self.threshold => {
        format!("{} {}%", emoji, percentage)
      }
      _ => String::new(),
    };
  }
}
