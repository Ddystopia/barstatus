use crate::Metric;
use std::fmt::Display;
use std::process::Command;
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemMetric {}

impl MemMetric {
    #[allow(dead_code)]
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Metric for MemMetric {
    fn timeout(&self) -> Duration {
        Duration::ZERO
    }
}

impl Display for MemMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: rewrite from shell api

        let out = Command::new("sh")
            .arg("-c")
            .arg("free -h | awk '/Mem/ {printf \"%s/%s\n\", $3, $2}'")
            .output()
            .map_err(|_| std::fmt::Error)?;

        let out = std::str::from_utf8(&out.stdout).map_err(|_| std::fmt::Error)?;

        write!(f, "📝 {out}")
    }
}
