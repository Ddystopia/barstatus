use std::{
    fmt::Display,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

use crate::{read_line::read_line_from_path, Metric};

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

struct NumFmt(u64);

impl Display for NumFmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pow = 0;
        let mut rem: u64 = 0;
        let mut number: u64 = self.0;
        while number > 1024 && pow < POWERS.len() - 1 {
            rem = number % 1024 * 100 / 1024;
            number /= 1024;
            pow += 1;
        }
        let power = POWERS[pow];

        if rem > 0 && number < 1000 {
            write!(f, "{number}.{rem}{power}")
        } else {
            write!(f, "{number}{power}")
        }
    }
}

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

    fn for_zipped_xfiles<F>(mut f: F)
    where
        F: for<'a> FnMut(&'a Path, &'a Path),
    {
        let Ok(out) = Command::new("sh")
            .arg("-c")
            .arg("ip addr | awk '/state UP/ {print $2}' | sed 's/.$//'")
            .output()
        else {
            return;
        };

        let Ok(ifs) = std::str::from_utf8(&out.stdout) else {
            return;
        };

        let mut ifs = ifs.lines().filter(|iface| !iface.is_empty());
        let paths = core::iter::from_fn(|| {
            let iface = ifs.next()?;
            let mut string = heapless::String::<256>::new();
            _ = string.push_str("/sys/class/net/");
            _ = string.push_str(iface);
            _ = string.push_str("/statistics/");
            let mut rx = string.clone();
            _ = rx.push_str("rx_bytes");
            _ = string.push_str("tx_bytes");

            Some((rx, string))
        });

        for (rx, tx) in paths {
            f(Path::new(&rx[..]), Path::new(&tx[..]));
        }
    }
}

impl Metric for NetMetric {
    fn timeout(&self) -> Duration {
        self.timeout
    }

    fn update(&mut self) {
        let delta = self.previous_update.elapsed();

        if delta < self.timeout {
            return;
        }

        let mut rx_bytes = 0;
        let mut tx_bytes = 0;

        Self::for_zipped_xfiles(|rx, tx| {
            let rx = read_line_from_path::<24>(rx);
            let tx = read_line_from_path::<24>(tx);
            let rx = rx.map(|rx| rx.parse::<u64>());
            let tx = tx.map(|tx| tx.parse::<u64>());
            if let (Ok(Ok(rx)), Ok(Ok(tx))) = (rx, tx) {
                rx_bytes += rx;
                tx_bytes += tx;
            }
        });

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
}

impl Display for NetMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "🔽{download}/s 🔼{upload}/s",
            download = NumFmt(self.download),
            upload = NumFmt(self.upload),
        )
    }
}
