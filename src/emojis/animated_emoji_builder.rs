use super::AnimatedEmoji;
use std::time::SystemTime;

use std::marker::PhantomData;

// The states to represent whether each field is set
pub struct MaxFrequencySet;
pub struct MaxFrequencyNotSet;
pub struct FramesSet;
pub struct FramesNotSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnimatedEmojiBuilder<MaxFrequencyState, FramesState> {
  max_frequency: Option<u32>,
  frames: Option<Vec<char>>,
  _phantom: PhantomData<(MaxFrequencyState, FramesState)>,
}

impl Default for AnimatedEmojiBuilder<MaxFrequencyNotSet, FramesNotSet> {
  fn default() -> AnimatedEmojiBuilder<MaxFrequencyNotSet, FramesNotSet> {
    AnimatedEmojiBuilder {
      max_frequency: None,
      frames: None,
      _phantom: PhantomData,
    }
  }
}

impl<FA, FB> AnimatedEmojiBuilder<FA, FB> {
  pub fn max_frequency(self, max_frequency: u32) -> AnimatedEmojiBuilder<MaxFrequencySet, FB> {
    assert!(max_frequency > 0);
    AnimatedEmojiBuilder {
      max_frequency: Some(max_frequency),
      frames: self.frames,
      _phantom: PhantomData,
    }
  }
  pub fn frames(self, frames: Vec<char>) -> AnimatedEmojiBuilder<FA, FramesSet> {
    assert!(!frames.is_empty());
    AnimatedEmojiBuilder {
      max_frequency: self.max_frequency,
      frames: Some(frames),
      _phantom: PhantomData,
    }
  }
}

impl AnimatedEmojiBuilder<MaxFrequencySet, FramesSet> {
  pub fn build(self) -> AnimatedEmoji {
    let max_frequency = self.max_frequency.unwrap();
    let frames = self.frames.unwrap();
    AnimatedEmoji::new(max_frequency, SystemTime::UNIX_EPOCH, frames)
  }
}
