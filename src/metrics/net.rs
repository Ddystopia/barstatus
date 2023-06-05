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
        let power = POWERS[pow];

        if rem > 0 && number < 1000 {
            format!("{number}.{rem}{power}")
        } else {
            format!("{number}{power}")
        }
    }
    fn get_zipped_xfiles() -> impl Iterator<Item = (File, File)> {
        Command::new("sh")
            .arg("-c")
            .arg("ip addr | awk '/state UP/ {print $2}' | sed 's/.$//'")
            .output()
            .ok()
            .into_iter()
            .flat_map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .split_whitespace()
                    .map(|interface| {
                        (
                            PathBuf::from("/sys/class/net/")
                                .join(interface)
                                .join("statistics/rx_bytes"),
                            PathBuf::from("/sys/class/net/")
                                .join(interface)
                                .join("statistics/tx_bytes"),
                        )
                    })
                    .filter(|(p1, p2)| p1.exists() && p2.exists())
                    .map(|(p1, p2)| (File::open(p1), File::open(p2)))
                    .filter_map(|(f1, f2)| Some((f1.ok()?, f2.ok()?)))
                    .collect::<Vec<_>>()
            })
    }
}

impl Metric for NetMetric {
    fn get_timeout(&self) -> Duration {
        self.timeout
    }
    fn update(&mut self) {
        let delta = match duration_since(self.previous_update) {
            Err(_) => return,
            Ok(d) if d < self.timeout => return,
            Ok(d) => d,
        };

        let (rx_bytes, tx_bytes) = NetMetric::get_zipped_xfiles()
            .filter_map(|(rx, tx)| Some((parse_xfile(rx)?, parse_xfile(tx)?)))
            .fold((0, 0), |(rx, tx), (r, t)| (rx + r, tx + t));

        let now = SystemTime::now();
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

    fn get_value(&mut self) -> Option<String> {
        Some(format!(
            "🔽{download}/s 🔼{upload}/s",
            download = NetMetric::numfmt(self.download),
            upload = NetMetric::numfmt(self.upload)
        ))
    }
}

fn parse_xfile(file: File) -> Option<u64> {
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    line.trim().parse().ok()
}
