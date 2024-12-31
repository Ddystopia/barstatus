use crate::{CommonError, Metric};
use std::{cell::Cell, fmt::Display};
use tokio::process::Command;

type Usage = heapless::String<24>;

#[derive(Default)]
pub struct MemMetric {
    usage: Cell<Usage>,
}

impl Metric for MemMetric {
    fn name(&self) -> &'static str {
        "Mem"
    }

    fn display(&self) -> impl Display {
        self
    }

    async fn update(&self) -> Result<(), CommonError> {
        match try {
            // TODO: rewrite from shell api
            let out = Command::new("sh")
                .arg("-c")
                .arg("free -h | awk '/Mem/ {printf \"%s/%s\n\", $3, $2}'")
                .output()
                .await?;

            let out = std::str::from_utf8(&out.stdout)?;

            self.usage
                .set(Usage::try_from(out).map_err(|()| CommonError::Capacity)?);
        } {
            Ok(()) => Ok(()),
            Err(err) => {
                self.usage.set(Default::default());
                Err(err)
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
