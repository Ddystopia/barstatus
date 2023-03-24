use super::animated_emoji_builder::{AnimatedEmojiBuilder, FramesNotSet, MaxCyclesNotSet};
use crate::duration_since;
use std::time::{Duration, SystemTime};

pub struct AnimatedEmoji {
  max_cycles_per_second: f32,
  frame: usize,
  previous_frame_update: SystemTime,
  frames: Vec<char>,
}

impl AnimatedEmoji {
  pub(super) fn new(
    max_cycles_per_second: f32,
    previous_frame_update: SystemTime,
    frames: Vec<char>,
  ) -> AnimatedEmoji {
    AnimatedEmoji {
      max_cycles_per_second,
      frame: 0,
      previous_frame_update,
      frames,
    }
  }

  pub fn builder() -> AnimatedEmojiBuilder<MaxCyclesNotSet, FramesNotSet> {
    AnimatedEmojiBuilder::<MaxCyclesNotSet, FramesNotSet>::default()
  }
  /// speed is a value between 0 and 1
  pub fn get_frame(&mut self, speed: f32) -> char {
    assert!((0.0..=1.0).contains(&speed));
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

  pub fn reset(&mut self) {
    self.frame = 0;
    self.previous_frame_update = SystemTime::UNIX_EPOCH;
  }
}
