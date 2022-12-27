use chrono::offset::Local;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader, Seek, SeekFrom};
use std::process::Command;
use std::time::{Duration, SystemTime};

pub trait Metric {
  fn update(&mut self) -> ();
  fn get_timeout(&self) -> Duration;
  fn get_value(&self) -> String;
}

pub struct CPUMetric {
  cpu_usage: u64,
  timeout: Duration,
  proc_file: fs::File,
  total: u64,
  idle: u64,
}

impl CPUMetric {
  pub fn new(timeout: Duration) -> CPUMetric {
    CPUMetric {
      cpu_usage: 0,
      timeout,
      proc_file: File::open("/proc/stat").expect("Failed to open proc stat file."),
      total: 1,
      idle: 1,
    }
  }
}

impl Metric for CPUMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn update(&mut self) -> () {
    let mut buf_reader = BufReader::new(&self.proc_file);
    buf_reader.seek(SeekFrom::Start(0_u64)).expect("/proc/stat");
    let mut timings = String::new();
    buf_reader.read_line(&mut timings).expect("/proc/stat");

    let timings = timings.split_whitespace().collect::<Vec<&str>>();
    let timings = vec![timings[1], timings[2], timings[3], timings[4]];
    let total = timings
      .iter()
      .map(|s| s.parse::<u64>().unwrap_or(1))
      .sum::<u64>();
    let idle = timings[3].parse::<u64>().unwrap_or(1);
    let delta_total = total - self.total;
    let delta_idle = idle - self.idle;
    self.cpu_usage = (delta_total * 100 - delta_idle * 100) / delta_total;
    self.total = total;
    self.idle = idle;
  }
  fn get_value(&self) -> String {
    format!("💻️ {:.0}% cpu", self.cpu_usage)
  }
}

pub struct NetMetric {
  timeout: Duration,
  upload: u64,
  download: u64,
  rx_bytes: u64,
  tx_bytes: u64,
  rx_files: Vec<File>,
  tx_files: Vec<File>,
  last_call: SystemTime,
}

impl NetMetric {
  pub fn new(timeout: Duration) -> NetMetric {
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
    let rx_files: Vec<File> = interfaces
      .iter()
      .map(|ifc| format!("/sys/class/net/{}/statistics/rx_bytes", ifc))
      .map(|path| File::open(path).expect("Failed to open network files"))
      .collect::<Vec<_>>();
    let tx_files: Vec<File> = interfaces
      .iter()
      .map(|ifc| format!("/sys/class/net/{}/statistics/tx_bytes", ifc))
      .map(|path| File::open(path).expect("Failed to open network files"))
      .collect::<Vec<_>>();

    NetMetric {
      timeout,
      upload: 0,
      download: 0,
      rx_bytes: 0,
      tx_bytes: 0,
      rx_files,
      tx_files,
      last_call: SystemTime::now(),
    }
  }
  fn numfmt(number: u64) -> String {
    let powers = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut pow = 0;
    let mut dec: u64 = 0;
    let mut number: u64 = number;
    while number > 1024 && pow < powers.len() - 1 {
      dec = number % 1024 * 100 / 1024;
      number /= 1024;
      pow += 1;
    }
    if dec > 0 && number <= 999 {
      format!("{}.{}{}", number, dec, powers[pow])
    } else {
      format!("{}{}", number, powers[pow])
    }
  }
}

impl Metric for NetMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn update(&mut self) -> () {
    let mut rx_bytes: u64 = 0;
    let mut tx_bytes: u64 = 0;

    for (rx, tx) in std::iter::zip(&self.rx_files, &self.tx_files) {
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
    let delta = now
      .duration_since(self.last_call)
      .expect("Net failed")
      .as_secs();

    if delta > 0 {
      self.download = (rx_bytes - self.rx_bytes) / delta;
      self.upload = (tx_bytes - self.tx_bytes) / delta;
    }
    self.rx_bytes = rx_bytes;
    self.tx_bytes = tx_bytes;
    self.last_call = now;
  }
  fn get_value(&self) -> String {
    format!(
      "🔽{}/s 🔼{}/s",
      NetMetric::numfmt(self.download),
      NetMetric::numfmt(self.upload)
    )
  }
}

pub struct DateMetric {
  timeout: Duration,
}

impl DateMetric {
  pub fn new(timeout: Duration) -> DateMetric {
    DateMetric { timeout }
  }
}

impl Metric for DateMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn update(&mut self) -> () {}
  fn get_value(&self) -> String {
    Local::now()
      .naive_local()
      .format("%a, %b %d %X")
      .to_string()
  }
}

pub struct BluetoothChargeMetric {
  timeout: Duration,
}

impl BluetoothChargeMetric {
  pub fn new(timeout: Duration) -> BluetoothChargeMetric {
    BluetoothChargeMetric { timeout }
  }
}

impl Metric for BluetoothChargeMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn update(&mut self) -> () {}
  fn get_value(&self) -> String {
    // TODO: rewrite from shell api
    let out = Command::new("sh")
      .arg("-c")
      .arg("bluetoothctl info | grep 'Battery Percentage' | sed 's/.*(\\([^)]*\\)).*/\\1/g'")
      .output()
      .expect("Failed to get bluetooth battery charge")
      .stdout;

    let percentage = String::from_utf8_lossy(&out).to_string();
    if percentage.len() > 0 {
      return format!("🎧⚡️ {}%", percentage);
    };
    return percentage;
  }
}

pub struct XkbLayoutMetric {
  timeout: Duration,
}

impl XkbLayoutMetric {
  pub fn new(timeout: Duration) -> XkbLayoutMetric {
    XkbLayoutMetric { timeout }
  }
}

impl Metric for XkbLayoutMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn update(&mut self) -> () {}
  fn get_value(&self) -> String {
    // TODO: rewrite from shell api
    let out = Command::new("sh")
      .arg("-c")
      .arg("xkb-switch")
      .output()
      .expect("Failed to get xkb layout")
      .stdout;
    let mut loc = String::from_utf8_lossy(&out).to_string();
    loc.pop();
    format!("🌍 {}", loc)
  }
}

pub struct UpdatesMetric {
  timeout: Duration,
  value: String,
}

impl UpdatesMetric {
  pub fn new(timeout: Duration) -> UpdatesMetric {
    UpdatesMetric {
      timeout,
      value: "".to_string(),
    }
  }
}

impl Metric for UpdatesMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn update(&mut self) -> () {
    let out = Command::new("sh")
      .arg("-c")
      .arg("checkupdates")
      .output()
      .expect("Failed to check updates")
      .stdout;
    let updates = String::from_utf8_lossy(&out).to_string();
    let system = if updates.contains("linux") { "!" } else { "" };
    let update_count = updates.lines().count();
    if update_count > 0 {
      self.value = format!("🔁 {}{}", system, update_count);
    }
  }
  fn get_value(&self) -> String {
    self.value.to_string()
  }
}

pub struct MemMetric {
  timeout: Duration,
}

impl MemMetric {
  #[allow(dead_code)]
  pub fn new(timeout: Duration) -> MemMetric {
    MemMetric { timeout }
  }
}

impl Metric for MemMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn update(&mut self) -> () {}
  fn get_value(&self) -> String {
    // TODO: rewrite from shell api
    let out = Command::new("sh")
      .arg("-c")
      .arg("free -h | awk '/Mem/ {printf \"%s/%s\n\", $3, $2}'")
      .output()
      .expect("Failed to get xkb layout")
      .stdout;
    format!("📝 {}", String::from_utf8_lossy(&out))
  }
}
