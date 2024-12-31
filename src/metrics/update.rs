use crate::Metric;
use std::cell::Cell;
use std::fmt::Display;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug)]
pub struct UpdatesMetric {
    system_update: Cell<bool>,
    updates_count: Cell<usize>,
    timeout: Duration,
}

impl UpdatesMetric {
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            updates_count: Cell::new(0),
            system_update: Cell::new(false),
        }
    }
}

impl Metric for UpdatesMetric {
    fn name(&self) -> &'static str {
        "Updates"
    }

    fn display(&self) -> impl Display {
        self
    }

    #[allow(clippy::unnecessary_map_or)]
    fn start(&self) -> impl std::future::Future<Output = !> + '_ {
        async {
            loop {
                let res: Option<(bool, usize)> = try {
                    let result = Command::new("sh")
                        .arg("-c")
                        .arg("checkupdates")
                        .output()
                        .await
                        .ok()?;

                    if !result.status.success() {
                        None?;
                    }

                    let updates = std::str::from_utf8(&result.stdout).ok()?;

                    self.system_update.set(updates.contains("linux"));
                    self.updates_count.set(updates.lines().count());

                    (updates.contains("linux"), updates.lines().count())
                };

                self.system_update.set(res.map_or(false, |(it, _)| it));
                self.updates_count.set(res.map_or(0, |(_, it)| it));

                tokio::time::sleep(self.timeout).await;
            }
        }
    }
}

impl Display for UpdatesMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let updates_count = self.updates_count.get();
        let system_update = self.system_update.get();

        if updates_count == 0 {
            return Ok(());
        }

        if system_update {
            write!(f, "🔁! {updates_count}")
        } else {
            write!(f, "🔁 {updates_count}")
        }
    }
}
