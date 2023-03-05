use crate::{duration_since, Metric};
use std::cell::Cell;
use std::process::Command;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

pub struct UpdatesMetric {
  system_update: Cell<bool>,  // cache
  updates_count: Cell<usize>, // cache
  timeout: Duration,
  receiver: Receiver<(usize, bool)>,
  stopper: Sender<bool>,
  update_thread: Option<JoinHandle<()>>,
}

impl UpdatesMetric {
  pub fn new(timeout: Duration) -> UpdatesMetric {
    let (tx_data, rx_data): (Sender<(usize, bool)>, Receiver<(usize, bool)>) = mpsc::channel();
    let (tx_stop, rx_stop): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let thread = thread::spawn(move || UpdatesMetric::updater(tx_data, rx_stop, timeout));

    UpdatesMetric {
      timeout,
      receiver: rx_data,
      stopper: tx_stop,
      update_thread: Some(thread),
      system_update: Cell::new(false),
      updates_count: Cell::new(0),
    }
  }

  fn updater(tx: Sender<(usize, bool)>, stopper: Receiver<bool>, timeout: Duration) {
    loop {
      if let Ok(true) = stopper.try_recv() {
        break;
      }

      let result = Command::new("sh")
        .arg("-c")
        .arg("checkupdates")
        .output()
        .expect("Failed to check updates");

      if !result.status.success() {
        tx.send((0, false))
      } else {
        let updates = String::from_utf8_lossy(&result.stdout).to_string();
        tx.send((updates.lines().count(), updates.contains("linux")))
      }
      .unwrap();

      thread::sleep(timeout);
    }
  }
}

impl Metric for UpdatesMetric {
  fn get_timeout(&self) -> Duration {
    self.timeout
  }

  fn update(&mut self) -> () {}

  fn get_value(&self) -> String {
    let (updates_count, system_update) = match self.receiver.try_recv() {
      Ok((0, _b)) => return String::new(),
      Ok((a, b)) => (a, b),
      Err(e) => match e {
        TryRecvError::Empty => (self.updates_count.get(), self.system_update.get()),
        TryRecvError::Disconnected => return String::new(),
      },
    };

    self.updates_count.set(updates_count);
    self.system_update.set(system_update);

    let sign = if system_update { "!" } else { "" };
    format!("🔁{} {}", sign, updates_count)
  }
}

impl Drop for UpdatesMetric {
  fn drop(&mut self) {
    if let Some(handle) = self.update_thread.take() {
      // Wait for thread to terminate
      self.stopper.send(true).unwrap();
      handle.join().unwrap();
    }
  }
}
