use crate::Metric;

use std::{cell::Cell, fmt::Display, time::Duration};
use tokio::process::Command;

type Locale = heapless::String<32>;

#[derive(Default)]
pub struct XkbLayoutMetric {
    timeout: Duration,
    locale: Cell<Option<Locale>>,
}

impl XkbLayoutMetric {
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            locale: Default::default(),
        }
    }
}

impl Metric for XkbLayoutMetric {
    fn name(&self) -> &'static str {
        "xkblayout"
    }

    fn display(&self) -> impl Display {
        self
    }

    fn start(&self) -> impl std::future::Future<Output = !> + '_ {
        async {
            loop {
                let loc: Option<Locale> = try {
                    let out = Command::new("sh")
                        .arg("-c")
                        .arg("xkb-switch")
                        .output()
                        .await
                        .ok()?;

                    let loc = std::str::from_utf8(&out.stdout).ok()?;

                    Locale::try_from(loc.strip_suffix('\n').unwrap_or(loc)).ok()?
                };

                self.locale.set(loc);

                tokio::time::sleep(self.timeout).await;
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
