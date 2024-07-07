use std::{
    cell::RefCell,
    fmt::Display,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{emojis::AnimatedEmoji, read_line::read_line_from_path, Metric};

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
const SLEEPING_THRESHOLD_PERCENTAGE: f64 = 0.1;
const SLEEPING_CAT_OLD: [char; 15] = range_chars('\u{e000}');
#[allow(dead_code)]
const RUNNING_CAT_OLD: [char; 5] = range_chars('\u{e100}');
#[allow(dead_code)]
const SLEEPING_CAT_NEW: [char; 15] = range_chars('\u{e200}');
const RUNNING_CAT_NEW: [char; 16] = range_chars('\u{e300}');

const MAX_FREQUENCY: f64 = 7.6;
const MIN_FREQUENCY: f64 = 0.5;

#[derive(Debug)]
pub struct CpuMetric {
    cpu_usage: Arc<AtomicU8>,
    should_stop: Arc<AtomicBool>,
    running_cat_emoji: RefCell<AnimatedEmoji<'static>>,
    sleeping_cat_emoji: RefCell<AnimatedEmoji<'static>>,
    timeout: Duration,
    handle: Option<JoinHandle<Result<(), ()>>>,
}

impl CpuMetric {
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        let cpu_usage = Arc::new(AtomicU8::new(0));
        let should_stop = Arc::new(AtomicBool::new(false));
        Self {
            cpu_usage: cpu_usage.clone(),
            should_stop: should_stop.clone(),
            timeout,
            running_cat_emoji: RefCell::new(
                AnimatedEmoji::builder()
                    .frames(RUNNING_CAT_NEW.as_slice())
                    .min_frequency(MIN_FREQUENCY)
                    .max_frequency(MAX_FREQUENCY)
                    .build(),
            ),
            sleeping_cat_emoji: RefCell::new(
                AnimatedEmoji::builder()
                    .frames(SLEEPING_CAT_OLD.as_slice())
                    .min_frequency(0.2)
                    .max_frequency(0.9)
                    .build(),
            ),
            handle: Some(thread::spawn(move || {
                Self::updater(cpu_usage, timeout, should_stop)
            })),
        }
    }

    fn get_emoji(&self) -> char {
        let cpu_usage = self.cpu_usage.load(Ordering::Relaxed) as f64 / 100.0;

        if cpu_usage < SLEEPING_THRESHOLD_PERCENTAGE {
            let speed = cpu_usage / SLEEPING_THRESHOLD_PERCENTAGE;
            self.running_cat_emoji.borrow_mut().reset();
            self.sleeping_cat_emoji.borrow_mut().next_frame(speed)
        } else {
            let speed = (cpu_usage - SLEEPING_THRESHOLD_PERCENTAGE) / (1. - SLEEPING_THRESHOLD_PERCENTAGE);
            self.sleeping_cat_emoji.borrow_mut().reset();
            self.running_cat_emoji.borrow_mut().next_frame(speed)
        }
    }

    fn updater(
        cpu_usage: Arc<AtomicU8>,
        timeout: Duration,
        should_stop: Arc<AtomicBool>,
    ) -> Result<(), ()> {
        let mut total_old: u64 = 1;
        let mut idle_old: u64 = 1;

        while !should_stop.load(Ordering::Relaxed) {
            let timings = read_line_from_path::<256>("/proc/stat").map_err(|_| ())?;

            let mut timings = timings
                .split_whitespace()
                .skip(1)
                .map(|s| s.parse().unwrap_or(0));

            let user = timings.next().expect("/proc/stat should have user");
            let nice = timings.next().expect("/proc/stat should have nice");
            let system = timings.next().expect("/proc/stat should have system");
            let idle = timings.next().expect("/proc/stat should have idle");
            let iowait = timings.next().expect("/proc/stat should have iowait");

            let total: u64 = user + nice + system + idle + iowait;
            let idle = idle + iowait;

            let delta_total = total.saturating_sub(total_old);
            let delta_idle = idle.saturating_sub(idle_old);
            let perc_u64 = (delta_total.saturating_sub(delta_idle) * 100)
                .checked_div(delta_total)
                .unwrap_or(0);

            let percentage = u8::try_from(perc_u64).expect("Always between 0 and 100");

            cpu_usage.store(percentage, Ordering::Relaxed);
            total_old = total;
            idle_old = idle;
            thread::sleep(timeout);
        }

        Ok(())
    }
}

impl Metric for CpuMetric {
    fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Display for CpuMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let emoji = self.get_emoji();
        let cpu_usage = self.cpu_usage.load(Ordering::Relaxed);
        write!(f, "{emoji} {cpu_usage: >2}% cpu")
    }
}

impl Drop for CpuMetric {
    fn drop(&mut self) {
        self.should_stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            // Wait for thread to terminate
            let _ = handle.join();
        }
    }
}
