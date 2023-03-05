use crate::{duration_since, Metric};
use std::fs::File;
use std::io::{prelude::*, BufReader, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

const SLEEPING_THRESHOLD_PERCENTAGE: u8 = 10;
const SLEEPING_CAT: char = '\u{e000}';
const MAX_CYCLES_PER_SECOND: f32 = 10.0; // 2.5
const RUNNING_CAT_FRAME_COUNT: usize = 5;
const RUNNING_CAT: [char; RUNNING_CAT_FRAME_COUNT] =
  ['\u{e001}', '\u{e002}', '\u{e003}', '\u{e004}', '\u{e005}'];

pub struct CPUMetric {
  cpu_usage: Arc<Mutex<u8>>,
  should_stop: Arc<AtomicBool>,
  current_running_cat: usize,
  timeout: Duration,
  previous_cat_update: SystemTime,
  handle: Option<JoinHandle<()>>,
}

impl CPUMetric {
  pub fn new(timeout: Duration) -> CPUMetric {
    let cpu_usage = Arc::new(Mutex::new(0 as u8));
    let should_stop = Arc::new(AtomicBool::new(false));
    CPUMetric {
      cpu_usage: cpu_usage.clone(),
      should_stop: should_stop.clone(),
      timeout,
      current_running_cat: 0,
      previous_cat_update: SystemTime::UNIX_EPOCH,
      handle: Some(thread::spawn(move || {
        CPUMetric::updater(cpu_usage, timeout, should_stop)
      })),
    }
  }
  fn get_emoji(&self) -> char {
    if *self.cpu_usage.lock().unwrap() < SLEEPING_THRESHOLD_PERCENTAGE {
      return SLEEPING_CAT;
    }
    RUNNING_CAT[self.current_running_cat]
  }
  fn update_running_cat_faze(&mut self) -> Option<()> {
    let cpu_usage = *self.cpu_usage.lock().unwrap();
    if cpu_usage < SLEEPING_THRESHOLD_PERCENTAGE {
      self.current_running_cat = 0;
      return Some(());
    }
    let cps = (cpu_usage as f32 / 100.0) * MAX_CYCLES_PER_SECOND;
    let fps = RUNNING_CAT_FRAME_COUNT as f32 * cps;
    let period_per_frame = Duration::from_millis((1000.0 / fps) as u64);
    if duration_since(self.previous_cat_update).ok()? < period_per_frame {
      return Some(());
    }
    self.previous_cat_update = SystemTime::now();
    self.current_running_cat += 1;
    self.current_running_cat %= RUNNING_CAT_FRAME_COUNT;
    Some(())
  }
  fn updater(cpu_usage: Arc<Mutex<u8>>, timeout: Duration, should_stop: Arc<AtomicBool>) {
    let mut total_old: u64 = 1;
    let mut idle_old: u64 = 1;

    loop {
      if should_stop.load(Ordering::Relaxed) {
        break;
      }
      let proc_file = File::open("/proc/stat").expect("Failed to open proc stat file.");
      let mut buf_reader = BufReader::new(proc_file);
      buf_reader.seek(SeekFrom::Start(0_u64)).expect("/proc/stat");
      let mut timings = String::new();
      buf_reader.read_line(&mut timings).expect("/proc/stat");

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

      *cpu_usage.lock().unwrap() = u8::try_from(perc_u64).unwrap(); // allways between 0 and 100
      total_old = total;
      idle_old = idle;
      thread::sleep(timeout);
    }
  }
}

impl Metric for CPUMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }
  fn get_value(&self) -> String {
    format!(
      "{} {: >2}% cpu",
      self.get_emoji(),
      self.cpu_usage.lock().unwrap()
    )
  }
  fn update(&mut self) {
    self.update_running_cat_faze();
  }
}

impl Drop for CPUMetric {
  fn drop(&mut self) {
    self.should_stop.store(true, Ordering::Relaxed);
    if let Some(handle) = self.handle.take() {
      // Wait for thread to terminate
      handle.join().unwrap();
    }
  }
}
