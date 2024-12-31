use std::{
    cell::Cell,
    fmt::{Display, Formatter},
};

use crate::{read_line::read_line_from_path, CommonError, Metric};

#[derive(Debug, Clone)]
pub struct BatteryMetric {
    threshold: u8,
    display: Cell<DisplayBattery>,
}

#[derive(Default, Debug, Clone, Copy)]
struct DisplayBattery(Option<&'static str>, Option<u8>, u8);

impl BatteryMetric {
    #[must_use]
    pub fn new(threshold: u8) -> Self {
        Self {
            threshold,
            display: Default::default(),
        }
    }

    async fn emoji(&self) -> Result<&'static str, CommonError> {
        Ok(
            match read_line_from_path::<24>("/sys/class/power_supply/BAT0/status").await? {
                status if status.trim() == "Charging" => "🔌🔼",
                status if status.trim() == "Discharging" => "🔋🔽",
                _ => "🔋",
            },
        )
    }

    async fn percentage(&self) -> Result<u8, CommonError> {
        let percentage = read_line_from_path::<24>("/sys/class/power_supply/BAT0/capacity");

        Ok(percentage.await?.trim().parse::<u8>()?)
    }
}

impl Metric for BatteryMetric {
    fn name(&self) -> &'static str {
        "Battery"
    }

    fn display(&self) -> impl Display {
        self.display.get()
    }

    async fn update(&self) -> Result<(), CommonError> {
        match try {
            self.display.set(DisplayBattery(
                Some(self.emoji().await?),
                Some(self.percentage().await?),
                self.threshold,
            ));
        } {
            Ok(()) => Ok(()),
            Err(err) => {
                self.display.set(DisplayBattery(None, None, self.threshold));
                Err(err)
            }
        }
    }
}

impl Display for DisplayBattery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Self(emoji, percentage, threeshold) = self;
        if let Some((emoji, percentage)) = emoji.zip(percentage) {
            if percentage < threeshold {
                write!(f, "{emoji} {percentage}%")?;
            }
        }

        Ok(())
    }
}
