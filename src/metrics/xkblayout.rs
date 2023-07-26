use crate::Metric;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XkbLayoutMetric {
    timeout: Duration,
}

impl XkbLayoutMetric {
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Metric for XkbLayoutMetric {
    fn get_timeout(&self) -> Duration {
        self.timeout
    }
    fn get_value(&self) -> Option<String> {
        // TODO: rewrite from shell api
        let out = Command::new("sh")
            .arg("-c")
            .arg("xkb-switch")
            .output()
            .ok()?;

        let loc = String::from_utf8_lossy(&out.stdout);

        Some(format!("🌍 {loc}", loc = &loc.strip_suffix('\n')?))
    }
}
