use crate::Metric;

use std::{cell::Cell, fmt::Display};
use tokio::process::Command;

type Locale = heapless::String<32>;

#[derive(Default)]
pub struct XkbLayoutMetric {
    locale: Cell<Option<Locale>>,
}

impl Metric for XkbLayoutMetric {
    fn name(&self) -> &'static str {
        "xkblayout"
    }

    fn display(&self) -> impl Display {
        self
    }

    async fn update(&self) -> Result<(), crate::CommonError> {
        match try {
            let out = Command::new("sh").arg("-c").arg("xkb-switch").output().await?;

            if !out.status.success() {
                return Err(crate::CommonError::UnsuccessfullShell(out.status));
            }

            let loc = std::str::from_utf8(&out.stdout)?;

            let locale = Locale::try_from(loc.strip_suffix('\n').unwrap_or(loc));
            self.locale.set(Some(locale.map_err(|()| crate::CommonError::Capacity)?));
        } {
            Ok(()) => Ok(()),
            Err(err) => {
                self.locale.set(None);
                Err(err)
            }
        }
    }
}

impl Display for XkbLayoutMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(locale) = self.locale.take() {
            let res = write!(f, "🌍 {locale}");
            self.locale.set(Some(locale));
            res
        } else {
            Ok(())
        }
    }
}
