use crate::Metric;

use std::cell::{Cell, RefCell};
use std::fmt::Display;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct XkbLayoutMetric {
    timeout: Duration,
    last_updated_at: Cell<Instant>,
    locale: RefCell<String>,
}

impl XkbLayoutMetric {
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            last_updated_at: Cell::new(Instant::now()),
            locale: RefCell::new(String::new()),
        }
    }

    fn update(&self) -> Result<(), std::fmt::Error> {
        let out = Command::new("sh")
            .arg("-c")
            .arg("xkb-switch")
            .output()
            .map_err(|_| std::fmt::Error)?;

        self.last_updated_at.set(Instant::now());

        let loc = std::str::from_utf8(&out.stdout).map_err(|_| std::fmt::Error)?;
        self.locale
            .replace(loc.strip_suffix('\n').unwrap_or(loc).to_string());
        Ok(())
    }
}

impl Metric for XkbLayoutMetric {
    fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Display for XkbLayoutMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.last_updated_at.get().elapsed() > self.timeout {
            self.update()?;
        }
        write!(f, "🌍 {loc}", loc = self.locale.borrow())
    }
}
