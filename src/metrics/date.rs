use crate::Metric;
use chrono::offset::Local;
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DateMetric;

impl Metric for DateMetric {
    fn name(&self) -> &'static str {
        "DateTime"
    }

    fn display(&self) -> impl Display {
        self
    }

    async fn update(&self) -> Result<(), !> {
        Ok(())
    }
}

impl Display for DateMetric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fmt = Local::now().naive_local().format("%a, %b %d %X");
        write!(f, "{fmt}")
    }
}
