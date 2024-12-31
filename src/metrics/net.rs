use std::{
    cell::Cell,
    fmt::Display,
    path::Path,
    time::{Duration, Instant},
};

use tokio::process::Command;

use crate::{read_line::read_line_from_path, CommonError, Metric};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct NetMetric(Cell<NetMetricInner>);

#[derive(Default, Copy, Debug, Clone, PartialEq, Eq)]
struct NetMetricInner {
    upload: u64,
    download: u64,
    rx_bytes: u64,
    tx_bytes: u64,
    previous_update: Option<Instant>,
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
    async fn for_zipped_xfiles<F: async FnMut(&Path, &Path)>(mut f: F) -> Result<(), CommonError> {
        let out = Command::new("sh")
            .arg("-c")
            .arg("ip addr | awk '/state UP/ {print $2}' | sed 's/.$//'")
            .output()
            .await?;

        let ifs = std::str::from_utf8(&out.stdout)?;
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
            f(Path::new(&rx[..]), Path::new(&tx[..])).await;
        }
        Ok(())
    }
}

impl Metric for NetMetric {
    fn display(&self) -> impl Display {
        self.0.get()
    }
    fn name(&self) -> &'static str {
        "Net"
    }

    async fn update(&self) -> Result<(), CommonError> {
        let mut inner = self.0.get();
        let delta = inner
            .previous_update
            .map_or(Duration::from_secs(0), |prev| prev.elapsed());

        let mut rx_bytes = 0;
        let mut tx_bytes = 0;

        Self::for_zipped_xfiles(async |rx, tx| {
            let rx = read_line_from_path::<24>(rx).await;
            let tx = read_line_from_path::<24>(tx).await;
            let rx = rx.map(|rx| rx.parse::<u64>());
            let tx = tx.map(|tx| tx.parse::<u64>());
            match (rx, tx) {
                (Ok(Ok(rx)), Ok(Ok(tx))) => {
                    rx_bytes += rx;
                    tx_bytes += tx;
                }
                (Ok(Err(err)), _) | (_, Ok(Err(err))) => {
                    log::warn!("Error parsing rx/tx bytes: {err}");
                }
                (Err(err), _) | (_, Err(err)) => {
                    log::warn!("Error reading rx/tx bytes: {err}");
                }
            }
        })
        .await?;

        let now = Instant::now();
        let delta = delta.as_secs();

        if delta > 0
            && rx_bytes > inner.rx_bytes
            && tx_bytes > inner.tx_bytes
            && inner.tx_bytes != 0
            && inner.rx_bytes != 0
        {
            inner.upload = (tx_bytes - inner.tx_bytes).checked_div(delta).unwrap_or(0);
            inner.download = (rx_bytes - inner.rx_bytes).checked_div(delta).unwrap_or(0);
        }

        inner.rx_bytes = rx_bytes;
        inner.tx_bytes = tx_bytes;
        inner.previous_update = Some(now);

        self.0.set(inner);

        Ok(())
    }
}

impl Display for NetMetricInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "🔽{download}/s 🔼{upload}/s",
            download = NumFmt(self.download),
            upload = NumFmt(self.upload),
        )
    }
}
