use crate::Metric;
use std::{cell::Cell, fmt::Display, time::Duration};
use tokio::process::Command;

type Usage = heapless::String<24>;

#[derive(Default)]
pub struct MemMetric {
    usage: Cell<Usage>,
}

impl MemMetric {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Metric for MemMetric {
    fn name(&self) -> &'static str {
        "Mem"
    }

    fn display(&self) -> impl Display {
        self
    }

    fn start(&self) -> impl std::future::Future<Output = !> + '_ {
        async move {
            loop {
                let usage: Option<Usage> = try {
                    // TODO: rewrite from shell api
                    let out = Command::new("sh")
                        .arg("-c")
                        .arg("free -h | awk '/Mem/ {printf \"%s/%s\n\", $3, $2}'")
                        .output()
                        .await
                        .ok()?;

                    let out = std::str::from_utf8(&out.stdout).ok()?;

                    Usage::try_from(out).ok()?
                };

                self.usage.set(usage.unwrap_or_default());

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

impl Display for MemMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let usage = self.usage.take();

        self.usage.set(usage.clone());

        write!(f, "📝 {usage}")
    }
}
