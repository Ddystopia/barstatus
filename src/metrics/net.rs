use crate::{duration_since, Metric};
use std::fs::File;
use std::io::{prelude::*, BufReader, Seek, SeekFrom};
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

impl NetMetric {
  pub fn new(timeout: Duration) -> NetMetric {
    NetMetric {
      timeout,
      upload: 0,
      download: 0,
      rx_bytes: 0,
      tx_bytes: 0,
      previous_update: SystemTime::now(),
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
    let interfaces = Command::new("sh")
      .arg("-c")
      .arg("ip addr | awk '/state UP/ {print $2}' | sed 's/.$//'")
      .output()
      .expect("Failed to get interfaces")
      .stdout;
    let interfaces = String::from_utf8_lossy(&interfaces)
      .split_whitespace()
      .map(|s| s.to_string())
      .collect::<Vec<_>>();

    interfaces
      .iter()
      .map(|interface| {
        (
          format!("/sys/class/net/{}/statistics/rx_bytes", interface),
          format!("/sys/class/net/{}/statistics/tx_bytes", interface),
        )
      })
      .map(|(p1, p2)| {
        (
          File::open(p1).expect("Failed to open network files"),
          File::open(p2).expect("Failed to open network files"),
        )
      })
      .collect::<Vec<_>>()
  }
}

impl Metric for NetMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn update(&mut self) -> () {
    let delta = match duration_since(self.previous_update) {
      Err(_) => return,
      Ok(d) => d,
    };

    if delta < self.timeout {
      return;
    }

    let mut rx_bytes: u64 = 0;
    let mut tx_bytes: u64 = 0;

    for (rx, tx) in NetMetric::get_zipped_xfiles() {
      let mut buf_reader = BufReader::new(rx);
      buf_reader.seek(SeekFrom::Start(0_u64)).expect("Net failed");
      let mut rx_b = String::new();
      buf_reader.read_line(&mut rx_b).expect("Net failed");

      let mut buf_reader = BufReader::new(tx);
      buf_reader.seek(SeekFrom::Start(0_u64)).expect("Net failed");
      let mut tx_b = String::new();
      buf_reader.read_line(&mut tx_b).expect("Net failed");

      rx_bytes += rx_b.trim().parse::<u64>().expect("Error rx");
      tx_bytes += tx_b.trim().parse::<u64>().expect("Error tx");
    }

    let now = SystemTime::now();
    let delta = delta.as_secs();

    if delta > 0 && rx_bytes > self.rx_bytes && tx_bytes > self.tx_bytes {
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
