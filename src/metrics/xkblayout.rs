use crate::Metric;

use std::cell::{Cell, RefCell};
use std::fmt::Display;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct XkbLayoutMetric {
    timeout: Duration,
}

impl XkbLayoutMetric {
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

    fn update(&self) -> Result<(), std::fmt::Error> {
        let out = Command::new("sh")
            .arg("-c")
            .arg("xkb-switch")
            .output()
            .ok()?;

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
