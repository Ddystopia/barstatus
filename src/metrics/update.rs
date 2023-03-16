use crate::Metric;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct UpdatesMetric {
  system_update: Arc<Mutex<bool>>,
  updates_count: Arc<Mutex<usize>>,
  should_stop: Arc<AtomicBool>,
  timeout: Duration,
  handle: Option<JoinHandle<()>>,
}

impl UpdatesMetric {
  pub fn new(timeout: Duration) -> UpdatesMetric {
    let updates_count = Arc::new(Mutex::new(0));
    let system_update = Arc::new(Mutex::new(false));
    let should_stop = Arc::new(AtomicBool::new(false));
    UpdatesMetric {
      timeout,
      updates_count: updates_count.clone(),
      system_update: system_update.clone(),
      should_stop: should_stop.clone(),
      handle: Some(thread::spawn(move || {
        UpdatesMetric::updater(system_update, updates_count, should_stop, timeout)
      })),
    }
  }
  fn updater(
    system_update: Arc<Mutex<bool>>,
    updates_count: Arc<Mutex<usize>>,
    should_stop: Arc<AtomicBool>,
    timeout: Duration,
  ) {
    loop {
      if should_stop.load(Ordering::Relaxed) {
        break;
      }

      let Ok(result) = Command::new("sh").arg("-c").arg("checkupdates").output() else {
        thread::sleep(timeout);
        continue;
      };

      if !result.status.success() {
        *updates_count.lock().unwrap() = 0;
        *system_update.lock().unwrap() = false;
      } else {
        let updates = String::from_utf8_lossy(&result.stdout).to_string();

        *updates_count.lock().unwrap() = updates.lines().count();
        *system_update.lock().unwrap() = updates.contains("linux");
      }
      thread::sleep(timeout);
    }
  }
}

impl Metric for UpdatesMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }

  fn update(&mut self) {}

  fn get_value(&self) -> String {
    let updates_count = *self.updates_count.lock().unwrap();
    let system_update = *self.system_update.lock().unwrap();
    if updates_count == 0 {
      return String::new();
    }
    let sign = if system_update { "!" } else { "" };
    format!("🔁{} {}", sign, updates_count)
  }
}

impl Drop for UpdatesMetric {
  fn drop(&mut self) {
    self.should_stop.store(true, Ordering::Relaxed);
    if let Some(handle) = self.handle.take() {
      // Wait for thread to terminate
      handle.join().unwrap_or(());
    }
  }
}
