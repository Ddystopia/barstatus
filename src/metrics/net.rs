use crate::{duration_since, Metric};
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetMetric {
  upload: u64,
  download: u64,
  rx_bytes: u64,
  tx_bytes: u64,
  timeout: Duration,
  previous_update: SystemTime,
}

const POWERS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

impl NetMetric {
  pub fn new(timeout: Duration) -> NetMetric {
    NetMetric {
      timeout,
      upload: 0,
      download: 0,
      rx_bytes: 0,
      tx_bytes: 0,
      previous_update: SystemTime::now() - timeout,
    }
  }
  fn numfmt(number: u64) -> String {
    let mut pow = 0;
    let mut rem: u64 = 0;
    let mut number: u64 = number;
    while number > 1024 && pow < POWERS.len() - 1 {
      rem = number % 1024 * 100 / 1024;
      number /= 1024;
      pow += 1;
    }
    if rem > 0 && number < 1000 {
      format!("{}.{}{}", number, rem, POWERS[pow])
    } else {
      format!("{}{}", number, POWERS[pow])
    }
  }
  fn get_zipped_xfiles() -> Vec<(File, File)> {
    let Ok(output) = Command::new("sh")
      .arg("-c")
      .arg("ip addr | awk '/state UP/ {print $2}' | sed 's/.$//'")
      .output() else
    {
      return Vec::new();
    };

    let interfaces = String::from_utf8_lossy(&output.stdout)
      .split_whitespace()
      .map(|s| s.to_string())
      .collect::<Vec<_>>();

    interfaces
      .into_iter()
      .map(|interface| {
        (
          PathBuf::from("/sys/class/net/")
            .join(&interface)
            .join("statistics/rx_bytes"),
          PathBuf::from("/sys/class/net/")
            .join(&interface)
            .join("statistics/tx_bytes"),
        )
      })
      .filter(|(p1, p2)| p1.exists() && p2.exists())
      .filter_map(|(p1, p2)| Some((File::open(p1).ok()?, File::open(p2).ok()?)))
      .collect()
  }
}

impl Metric for NetMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn update(&mut self) {
    let delta = match duration_since(self.previous_update) {
      Err(_) => return,
      Ok(d) => d,
    };

    if delta < self.timeout {
      return;
    }

    let (rx_bytes, tx_bytes) = NetMetric::get_zipped_xfiles()
      .into_iter()
      .filter_map(|(rx, tx)| Some((parse_xfile(rx)?, parse_xfile(tx)?)))
      .fold((0, 0), |(rx, tx), (rx1, tx1)| (rx + rx1, tx + tx1));

    let now = SystemTime::now();
    let delta = delta.as_secs();

    if delta > 0
      && rx_bytes > self.rx_bytes
      && tx_bytes > self.tx_bytes
      && self.tx_bytes != 0
      && self.rx_bytes != 0
    {
      self.upload = (tx_bytes - self.tx_bytes) / delta;
      self.download = (rx_bytes - self.rx_bytes) / delta;
    }

    self.rx_bytes = rx_bytes;
    self.tx_bytes = tx_bytes;
    self.previous_update = now;
  }
  fn get_value(&mut self) -> String {
    format!(
      "🔽{}/s 🔼{}/s",
      NetMetric::numfmt(self.download),
      NetMetric::numfmt(self.upload)
    )
  }
}

fn parse_xfile(file: File) -> Option<u64> {
  let mut reader = BufReader::new(file);
  let mut line = String::new();
  reader.read_line(&mut line).ok()?;
  line.trim().parse().ok()
}
