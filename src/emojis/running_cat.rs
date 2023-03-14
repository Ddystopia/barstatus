use std::{
  sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    Arc,
  },
  thread::{self, JoinHandle},
};

use std::time::Duration;

pub struct RunningCat {
  frame: Arc<AtomicU8>,
  max_cycles_per_second: f32,
  should_stop: Arc<AtomicBool>,
  frames: Vec<char>,
  handle: Option<JoinHandle<()>>,
}

impl RunningCat {
  pub fn new(max_cycles_per_second: f32, frames: Vec<char>) -> RunningCat {
    let frame = Arc::new(AtomicU8::new(0));
    let should_stop = Arc::new(AtomicBool::new(false));
    let _fps = max_cycles_per_second * frames.len() as f32;
    RunningCat {
      max_cycles_per_second,
      frames,
      frame,
      should_stop,
      handle: Some(thread::spawn(|| {})),
    }
  }
  fn updater(should_stop: Arc<AtomicBool>) {
    loop {
      if should_stop.load(Ordering::Relaxed) {
        break;
      }
      thread::sleep(Duration::from_millis(10));
    }
  }
}
