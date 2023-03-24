use super::AnimatedEmoji;
use std::time::SystemTime;

use std::marker::PhantomData;

// The states to represent whether each field is set
pub struct MaxCyclesSet;
pub struct MaxCyclesNotSet;
pub struct FramesSet;
pub struct FramesNotSet;

pub struct AnimatedEmojiBuilder<MaxCyclesState, FramesState> {
  max_cycles_per_second: Option<f32>,
  frames: Option<Vec<char>>,
  _phantom: PhantomData<(MaxCyclesState, FramesState)>,
}

impl Default for AnimatedEmojiBuilder<MaxCyclesNotSet, FramesNotSet> {
  fn default() -> AnimatedEmojiBuilder<MaxCyclesNotSet, FramesNotSet> {
    AnimatedEmojiBuilder {
      max_cycles_per_second: None,
      frames: None,
      _phantom: PhantomData,
    }
  }
}

impl<FA, FB> AnimatedEmojiBuilder<FA, FB> {
  pub fn max_cycles_per_second(
    self,
    max_cycles_per_second: f32,
  ) -> AnimatedEmojiBuilder<MaxCyclesSet, FB> {
    assert!(max_cycles_per_second > 0.);
    AnimatedEmojiBuilder {
      max_cycles_per_second: Some(max_cycles_per_second),
      frames: self.frames,
      _phantom: PhantomData,
    }
  }
  pub fn frames(self, frames: Vec<char>) -> AnimatedEmojiBuilder<FA, FramesSet> {
    assert!(!frames.is_empty());
    AnimatedEmojiBuilder {
      max_cycles_per_second: self.max_cycles_per_second,
      frames: Some(frames),
      _phantom: PhantomData,
    }
  }
}

impl AnimatedEmojiBuilder<MaxCyclesSet, FramesSet> {
  pub fn build(self) -> AnimatedEmoji {
    let max_cycles_per_second = self.max_cycles_per_second.unwrap();
    let frames = self.frames.unwrap();
    AnimatedEmoji::new(max_cycles_per_second, SystemTime::UNIX_EPOCH, frames)
  }
}
