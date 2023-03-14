use crate::{duration_since, Metric};
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};

pub struct NetMetric {
  upload: u64,
  download: u64,
  rx_bytes: u64,
  tx_bytes: u64,
  timeout: Duration,
  previous_update: SystemTime,
}
const POWERS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

macro_rules! skip_fail {
  ($res:expr) => {
    match $res {
      Ok(val) => val,
      Err(e) => {
        eprintln!("An error: {}; skipped.", e);
        continue;
      }
    }
  };
}

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
    let interfaces = match Command::new("sh")
      .arg("-c")
      .arg("ip addr | awk '/state UP/ {print $2}' | sed 's/.$//'")
      .output()
    {
      Ok(output) => output.stdout,
      Err(_) => return Vec::new(),
    };

    let interfaces = String::from_utf8_lossy(&interfaces)
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

    let mut rx_bytes: u64 = 0;
    let mut tx_bytes: u64 = 0;

    for (mut rx, mut tx) in NetMetric::get_zipped_xfiles()
      .into_iter()
      .map(|(r, t)| (BufReader::new(r), BufReader::new(t)))
    {
      let mut rx_b = String::new();
      let mut tx_b = String::new();

      skip_fail!(rx.read_line(&mut rx_b));
      skip_fail!(tx.read_line(&mut tx_b));

      rx_bytes += skip_fail!(rx_b.trim().parse::<u64>());
      tx_bytes += skip_fail!(tx_b.trim().parse::<u64>());
    }

    let now = SystemTime::now();
    let delta = delta.as_secs();

    if delta > 0
      && rx_bytes > self.rx_bytes
      && tx_bytes > self.tx_bytes
      && self.tx_bytes != 0
      && self.rx_bytes != 0
    {
      self.download = (rx_bytes - self.rx_bytes) / delta;
      self.upload = (tx_bytes - self.tx_bytes) / delta;
    }

    self.rx_bytes = rx_bytes;
    self.tx_bytes = tx_bytes;
    self.previous_update = now;
  }
  fn get_value(&self) -> String {
    format!(
      "🔽{}/s 🔼{}/s",
      NetMetric::numfmt(self.download),
      NetMetric::numfmt(self.upload)
    )
  }
}
