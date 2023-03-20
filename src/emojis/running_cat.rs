use crate::duration_since;
use std::time::{Duration, SystemTime};

#[derive(Default)]
pub struct RunningCatBuilder {
  max_cycles_per_second: f32,
  speed_threshold: f32,
  frames: Vec<char>,
  sleep_frame: char,
}

impl RunningCatBuilder {
  pub fn build(self) -> RunningCat {
    RunningCat {
      max_cycles_per_second: self.max_cycles_per_second,
      speed_threshold: self.speed_threshold,
      frames: self.frames,
      sleep_frame: self.sleep_frame,
      frame: 0,
      previous_frame_update: SystemTime::UNIX_EPOCH,
    }
  }
  pub fn max_cycles_per_second(mut self, max_cycles_per_second: f32) -> Self {
    assert!(max_cycles_per_second > 0.);
    self.max_cycles_per_second = max_cycles_per_second;
    self
  }
  pub fn speed_threshold(mut self, speed_threshold: f32) -> Self {
    assert!((0.0..=1.0).contains(&speed_threshold));
    self.speed_threshold = speed_threshold;
    self
  }
  pub fn frames(mut self, frames: Vec<char>) -> Self {
    assert!(!frames.is_empty());
    self.frames = frames;
    self
  }
  pub fn sleep_frame(mut self, sleep_frame: char) -> Self {
    self.sleep_frame = sleep_frame;
    self
  }
}

pub struct RunningCat {
  max_cycles_per_second: f32,
  speed_threshold: f32,
  frame: usize,
  previous_frame_update: SystemTime,
  frames: Vec<char>,
  sleep_frame: char,
}

impl RunningCat {
  pub fn builder() -> RunningCatBuilder {
    RunningCatBuilder::default()
  }
  /// speed is a value between 0 and 1
  pub fn get_frame(&mut self, speed: f32) -> char {
    assert!((0.0..=1.0).contains(&speed));
    if speed < self.speed_threshold {
      return self.sleep_frame;
    }
    let cycles_per_second = speed * self.max_cycles_per_second;
    let fps = self.frames.len() as f32 * cycles_per_second;
    let period_per_frame = Duration::from_millis((1000.0 / fps) as u64);

    if let Ok(previous_update) = duration_since(self.previous_frame_update) {
      if previous_update > period_per_frame {
        self.previous_frame_update = SystemTime::now();
        self.frame += 1;
        self.frame %= self.frames.len();
      }
    }

    self.frames[self.frame]
  }
}
