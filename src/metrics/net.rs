use crate::Metric;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetMetric {
    upload: u64,
    download: u64,
    rx_bytes: u64,
    tx_bytes: u64,
    timeout: Duration,
    previous_update: Instant,
}

const POWERS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

impl NetMetric {
    /// Creates a new `NetMetric` with the given timeout.
    /// # Panics
    /// If the timeout is too large.
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            upload: 0,
            download: 0,
            rx_bytes: 0,
            tx_bytes: 0,
            previous_update: Instant::now().checked_sub(timeout).unwrap(),
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
        let power = POWERS[pow];

        if rem > 0 && number < 1000 {
            format!("{number}.{rem}{power}")
        } else {
            format!("{number}{power}")
        }
    }

    fn get_zipped_xfiles() -> Vec<(File, File)> {
        Command::new("sh")
            .arg("-c")
            .arg("ip addr | awk '/state UP/ {print $2}' | sed 's/.$//'")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .unwrap_or_default()
            .split_whitespace()
            .map(|iface| PathBuf::from("/sys/class/net/").join(iface))
            .map(|iface| iface.join("statistics/"))
            .map(|iface| (iface.join("rx_bytes"), iface.join("tx_bytes")))
            .filter(|(p1, p2)| p1.exists() && p2.exists())
            .map(|(p1, p2)| (File::open(p1), File::open(p2)))
            .filter_map(|(f1, f2)| Some((f1.ok()?, f2.ok()?)))
            .collect()
    }
}

impl Metric for NetMetric {
    fn get_timeout(&self) -> Duration {
        self.timeout
    }

    fn update(&mut self) {
        let delta = self.previous_update.elapsed();

        if delta < self.timeout {
            return;
        }

        let (rx_bytes, tx_bytes) = Self::get_zipped_xfiles()
            .into_iter()
            .filter_map(|(rx, tx)| Some((parse_xfile(rx)?, parse_xfile(tx)?)))
            .fold((0, 0), |(rx, tx), (r, t)| (rx + r, tx + t));

        let now = Instant::now();
        let delta = delta.as_secs();

        if delta > 0
            && rx_bytes > self.rx_bytes
            && tx_bytes > self.tx_bytes
            && self.tx_bytes != 0
            && self.rx_bytes != 0
        {
            self.upload = (tx_bytes - self.tx_bytes).checked_div(delta).unwrap_or(0);
            self.download = (rx_bytes - self.rx_bytes).checked_div(delta).unwrap_or(0);
        }

        self.rx_bytes = rx_bytes;
        self.tx_bytes = tx_bytes;
        self.previous_update = now;
    }

    fn get_value(&self) -> Option<String> {
        Some(format!(
            "🔽{download}/s 🔼{upload}/s",
            download = Self::numfmt(self.download),
            upload = Self::numfmt(self.upload)
        ))
    }
}

fn parse_xfile(file: File) -> Option<u64> {
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    line.trim().parse().ok()
}
