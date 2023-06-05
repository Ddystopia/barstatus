use crate::emojis::AnimatedEmoji;
use crate::Metric;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const SLEEPING_THRESHOLD_PERCENTAGE: f32 = 0.1;
const SLEEPING_CAT: char = '\u{e000}';
const MAX_FREQUENCY: u32 = 5;
const RUNNING_CAT_FRAME_COUNT: usize = 5;
const RUNNING_CAT: [char; RUNNING_CAT_FRAME_COUNT] =
    ['\u{e001}', '\u{e002}', '\u{e003}', '\u{e004}', '\u{e005}'];

#[derive(Debug)]
pub struct CpuMetric {
    cpu_usage: Arc<AtomicU8>,
    should_stop: Arc<AtomicBool>,
    running_cat_emoji: AnimatedEmoji<'static>,
    sleeping_cat_emoji: AnimatedEmoji<'static>,
    timeout: Duration,
    handle: Option<JoinHandle<Result<(), ()>>>,
}

impl CpuMetric {
    pub fn new(timeout: Duration) -> CpuMetric {
        let cpu_usage = Arc::new(AtomicU8::new(0));
        let should_stop = Arc::new(AtomicBool::new(false));
        CpuMetric {
            cpu_usage: cpu_usage.clone(),
            should_stop: should_stop.clone(),
            timeout,
            running_cat_emoji: AnimatedEmoji::builder()
                .frames(RUNNING_CAT.as_slice())
                .max_frequency(MAX_FREQUENCY)
                .build(),
            sleeping_cat_emoji: AnimatedEmoji::builder()
                .frames([SLEEPING_CAT].as_slice())
                .max_frequency(MAX_FREQUENCY)
                .build(),
            handle: Some(thread::spawn(move || {
                CpuMetric::updater(cpu_usage, timeout, should_stop)
            })),
        }
    }

    fn get_emoji(&mut self) -> char {
        let cpu_usage = self.cpu_usage.load(Ordering::Relaxed) as f32 / 100.0;
        if cpu_usage < SLEEPING_THRESHOLD_PERCENTAGE {
            self.running_cat_emoji.reset();
            self.sleeping_cat_emoji.get_frame(cpu_usage)
        } else {
            self.sleeping_cat_emoji.reset();
            self.running_cat_emoji.get_frame(cpu_usage)
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
            let proc_file = File::open("/proc/stat").map_err(|_| ())?;
            let mut buf_reader = BufReader::new(proc_file);
            let mut timings = String::new();
            buf_reader.read_line(&mut timings).map_err(|_| ())?;

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
    fn get_timeout(&self) -> Duration {
        self.timeout
    }
    fn get_value(&mut self) -> Option<String> {
        Some(format!(
            "{emoji} {cpu_usage: >2}% cpu",
            emoji = self.get_emoji(),
            cpu_usage = self.cpu_usage.load(Ordering::Relaxed),
        ))
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
