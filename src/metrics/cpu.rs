use std::{
    cell::{Cell, RefCell},
    fmt::Display,
    future::Future,
    time::Duration,
};

use crate::{emojis::AnimatedEmoji, read_line::read_line_from_path, Metric};

mod emojis {
    #![allow(dead_code)]

    const fn range_chars<const N: usize>(base: char) -> [char; N] {
        let mut r = ['0'; N];
        let mut i = 0;
        while i < N {
            r[i] = match core::char::from_u32(base as u32 + i as u32) {
                Some(c) => c,
                None => panic!(),
            };
            i += 1;
        }
        r
    }

    pub const SLEEPING_CAT_OLD: [char; 15] = range_chars('\u{e000}');
    pub const RUNNING_CAT_OLD: [char; 5] = range_chars('\u{e100}');
    pub const SLEEPING_CAT_NEW: [char; 15] = range_chars('\u{e200}');
    pub const RUNNING_CAT_NEW: [char; 16] = range_chars('\u{e300}');
}

const SLEEPING_THRESHOLD_PERCENTAGE: f64 = 0.1;

const MAX_FREQUENCY: f64 = 7.6;
const MIN_FREQUENCY: f64 = 0.5;

#[derive(Debug)]
pub struct CpuMetric {
    cpu_usage: Cell<Option<u8>>,
    total: Cell<u64>,
    idle: Cell<u64>,
    running_cat_emoji: RefCell<AnimatedEmoji<'static>>,
    sleeping_cat_emoji: RefCell<AnimatedEmoji<'static>>,
    timeout: Duration,
}

impl CpuMetric {
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self {
            cpu_usage: Default::default(),
            total: Cell::new(1),
            idle: Cell::new(1),
            timeout,
            running_cat_emoji: RefCell::new(
                AnimatedEmoji::builder()
                    .frames(emojis::RUNNING_CAT_NEW.as_slice())
                    .min_frequency(MIN_FREQUENCY)
                    .max_frequency(MAX_FREQUENCY)
                    .build(),
            ),
            sleeping_cat_emoji: RefCell::new(
                AnimatedEmoji::builder()
                    .frames(emojis::SLEEPING_CAT_OLD.as_slice())
                    .min_frequency(0.2)
                    .max_frequency(0.9)
                    .build(),
            ),
        }
    }

    fn get_emoji(&self, cpu_usage: u8) -> char {
        let cpu_usage = cpu_usage as f64 / 100.0;
        let threshold = SLEEPING_THRESHOLD_PERCENTAGE;

        if cpu_usage < threshold {
            let speed = cpu_usage / threshold;
            self.running_cat_emoji.borrow_mut().reset();
            self.sleeping_cat_emoji.borrow_mut().next_frame(speed)
        } else {
            let speed = (cpu_usage - threshold) / (1. - threshold);
            self.sleeping_cat_emoji.borrow_mut().reset();
            self.running_cat_emoji.borrow_mut().next_frame(speed)
        }
    }

    async fn read_percentage(&self) -> Option<u8> {
        let timings = read_line_from_path::<256>("/proc/stat").await.ok()?;

        let mut timings = timings
            .split_whitespace()
            .skip(1)
            .map(|s| s.parse().unwrap_or(0));

        let user = timings.next()?;
        let nice = timings.next()?;
        let system = timings.next()?;
        let idle_ = timings.next()?;
        let iowait = timings.next()?;

        let total_new: u64 = user + nice + system + idle_ + iowait;
        let idle_new = idle_ + iowait;

        let delta_total = total_new.saturating_sub(self.total.get());
        let delta_idle = idle_new.saturating_sub(self.idle.get());
        let perc_u64 = (delta_total.saturating_sub(delta_idle) * 100)
            .checked_div(delta_total)
            .unwrap_or(0);

        let percentage = u8::try_from(perc_u64).ok()?;

        self.total.set(total_new);
        self.idle.set(idle_new);

        Some(percentage)
    }
}

impl Metric for CpuMetric {
    fn name(&self) -> &'static str {
        "CPU"
    }

    fn display(&self) -> impl Display {
        self
    }

    fn start(&self) -> impl Future<Output = !> + '_ {
        async {
            loop {
                let percentage = self.read_percentage().await;

                self.cpu_usage.set(percentage);

                tokio::time::sleep(self.timeout).await;
            }
        }
    }
}

impl Display for CpuMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cpu_usage = self.cpu_usage.get();

        if let Some(cpu_usage) = cpu_usage {
            let emoji = self.get_emoji(cpu_usage);

            write!(f, "{emoji} {cpu_usage: >2}% cpu")
        } else {
            Ok(())
        }
    }
}
