use crate::Metric;
use std::fmt::Display;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug)]
pub struct UpdatesMetric {
    system_update: Arc<AtomicBool>,
    updates_count: Arc<AtomicUsize>,
    should_stop: Arc<AtomicBool>,
    timeout: Duration,
    handle: Option<JoinHandle<()>>,
}

impl UpdatesMetric {
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        let updates_count = Arc::new(AtomicUsize::new(0));
        let system_update = Arc::new(AtomicBool::new(false));
        let should_stop = Arc::new(AtomicBool::new(false));
        Self {
            timeout,
            updates_count: updates_count.clone(),
            system_update: system_update.clone(),
            should_stop: should_stop.clone(),
            handle: Some(thread::spawn(move || {
                Self::updater(system_update, updates_count, should_stop, timeout);
            })),
        }
    }
    fn updater(
        system_update: Arc<AtomicBool>,
        updates_count: Arc<AtomicUsize>,
        should_stop: Arc<AtomicBool>,
        timeout: Duration,
    ) {
        while !should_stop.load(Ordering::Relaxed) {
            let Ok(result) = Command::new("sh").arg("-c").arg("checkupdates").output() else {
                thread::sleep(timeout);
                continue;
             };

            if result.status.success() {
                let updates = String::from_utf8_lossy(&result.stdout).to_string();

                updates_count.store(updates.lines().count(), Ordering::Relaxed);
                system_update.store(updates.contains("linux"), Ordering::Relaxed);
            } else {
                updates_count.store(0, Ordering::Relaxed);
                system_update.store(false, Ordering::Relaxed);
            }
            thread::sleep(timeout);
        }
    }
}

impl Metric for UpdatesMetric {
    fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Display for UpdatesMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let updates_count = self.updates_count.load(Ordering::Relaxed);
        let system_update = self.system_update.load(Ordering::Relaxed);

        if updates_count == 0 {
            return Ok(());
        }

        if system_update {
            write!(f, "🔁! {}", updates_count)
        } else {
            write!(f, "🔁 {}", updates_count)
        }
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
