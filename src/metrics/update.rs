use crate::{CommonError, Metric};
use std::{cell::Cell, fmt::Display};
use tokio::process::Command;

#[derive(Debug, Default)]
pub struct UpdatesMetric {
    system_update: Cell<bool>,
    updates_count: Cell<usize>,
}

impl Metric for UpdatesMetric {
    fn name(&self) -> &'static str {
        "Updates"
    }

    fn display(&self) -> impl Display {
        self
    }

    #[allow(clippy::unnecessary_map_or)]
    async fn update(&self) -> Result<(), CommonError> {
        match try {
            let result = Command::new("sh")
                .arg("-c")
                .arg("checkupdates")
                .output()
                .await?;

            if !result.status.success() {
                return Err(CommonError::UnsuccessfullShell(result.status));
            }

            let updates = std::str::from_utf8(&result.stdout)?;

            self.system_update.set(updates.contains("linux"));
            self.updates_count.set(updates.lines().count());
        } {
            Ok(()) => Ok(()),
            Err(err) => {
                self.system_update.set(false);
                self.updates_count.set(0);
                Err(err)
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
