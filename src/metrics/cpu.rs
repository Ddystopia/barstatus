use crate::emojis::RunningCat;
use crate::Metric;
use std::fs::File;
use std::io::{prelude::*, BufReader, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const SLEEPING_THRESHOLD_PERCENTAGE: u8 = 10;
const SLEEPING_CAT: char = '\u{e000}';
const MAX_CYCLES_PER_SECOND: f32 = 10.0; // 2.5
const RUNNING_CAT_FRAME_COUNT: usize = 5;
const RUNNING_CAT: [char; RUNNING_CAT_FRAME_COUNT] =
  ['\u{e001}', '\u{e002}', '\u{e003}', '\u{e004}', '\u{e005}'];

macro_rules! skip_fail {
  ($res:expr) => {
    match $res {
      Ok(val) => val,
      Err(e) => {
        eprintln!("An error: {}; skipped.", e);
        continue;
      }
    }
  };
}

pub struct CpuMetric {
  cpu_usage: Arc<AtomicU8>,
  should_stop: Arc<AtomicBool>,
  cat_emoji: RunningCat,
  timeout: Duration,
  handle: Option<JoinHandle<()>>,
}

impl CpuMetric {
  pub fn new(timeout: Duration) -> CpuMetric {
    let cpu_usage = Arc::new(AtomicU8::new(0));
    let should_stop = Arc::new(AtomicBool::new(false));
    CpuMetric {
      cpu_usage: cpu_usage.clone(),
      should_stop: should_stop.clone(),
      timeout,
      cat_emoji: RunningCat::builder()
        .frames(RUNNING_CAT.to_vec())
        .sleep_frame(SLEEPING_CAT)
        .speed_threshold(SLEEPING_THRESHOLD_PERCENTAGE as f32 / 100.)
        .max_cycles_per_second(MAX_CYCLES_PER_SECOND)
        .build(),
      handle: Some(thread::spawn(move || {
        CpuMetric::updater(cpu_usage, timeout, should_stop)
      })),
    }
  }
  fn get_emoji(&mut self) -> char {
    let cpu_usage = self.cpu_usage.load(Ordering::Relaxed);
    self.cat_emoji.get_frame(cpu_usage as f32 / 100.)
  }

  fn updater(cpu_usage: Arc<AtomicU8>, timeout: Duration, should_stop: Arc<AtomicBool>) {
    let mut total_old: u64 = 1;
    let mut idle_old: u64 = 1;

    loop {
      if should_stop.load(Ordering::Relaxed) {
        break;
      }
      let proc_file = skip_fail!(File::open("/proc/stat"));
      let mut buf_reader = BufReader::new(proc_file);
      skip_fail!(buf_reader.seek(SeekFrom::Start(0_u64)));
      let mut timings = String::new();
      skip_fail!(buf_reader.read_line(&mut timings));

      let timings = timings.split_whitespace().collect::<Vec<&str>>();
      let timings = vec![timings[1], timings[2], timings[3], timings[4]];
      let total = timings
        .iter()
        .map(|s| s.parse::<u64>().unwrap_or(1))
        .sum::<u64>();
      let idle = timings[3].parse::<u64>().unwrap_or(1);
      let delta_total = total - total_old;
      let delta_idle = idle - idle_old;
      let perc_u64 = (delta_total * 100 - delta_idle * 100) / delta_total;
      let percentage = u8::try_from(perc_u64).unwrap_or(1); // allways between 0 and 100

      cpu_usage.store(percentage, Ordering::Relaxed);
      total_old = total;
      idle_old = idle;
      thread::sleep(timeout);
    }
  }
}

impl Metric for CpuMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn get_value(&mut self) -> String {
    format!(
      "{} {: >2}% cpu",
      self.get_emoji(),
      self.cpu_usage.load(Ordering::Relaxed),
    )
  }
  fn update(&mut self) {}
}

impl Drop for CpuMetric {
  fn drop(&mut self) {
    self.should_stop.store(true, Ordering::Relaxed);
    if let Some(handle) = self.handle.take() {
      // Wait for thread to terminate
      handle.join().unwrap_or(());
    }
  }
}
