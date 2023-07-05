use crate::Metric;
use std::process::Command;
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemMetric {}

impl MemMetric {
    #[allow(dead_code)]
    pub fn new() -> MemMetric {
        MemMetric {}
    }
}

impl Metric for MemMetric {
    fn get_timeout(&self) -> Duration {
        Duration::ZERO
    }
    fn get_value(&self) -> Option<String> {
        // TODO: rewrite from shell api
        let out = Command::new("sh")
            .arg("-c")
            .arg("free -h | awk '/Mem/ {printf \"%s/%s\n\", $3, $2}'")
            .output()
            .ok()?;

        Some(format!("📝 {}", String::from_utf8_lossy(&out.stdout)))
    }
}
