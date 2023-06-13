use super::AnimatedEmoji;
use std::time::SystemTime;

// The states to represent whether each field is set
pub struct MaxFrequencySet(u32);
pub struct MaxFrequencyNotSet;
pub struct FramesSet<'a>(&'a [char]);
pub struct FramesNotSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnimatedEmojiBuilder<MaxFrequencyState = MaxFrequencyNotSet, FramesState = FramesNotSet>
{
    max_frequency: MaxFrequencyState,
    frames: FramesState,
}

impl Default for AnimatedEmojiBuilder<MaxFrequencyNotSet, FramesNotSet> {
    fn default() -> AnimatedEmojiBuilder<MaxFrequencyNotSet, FramesNotSet> {
        AnimatedEmojiBuilder {
            max_frequency: MaxFrequencyNotSet,
            frames: FramesNotSet,
        }
    }
}

impl<'a, FA, FB> AnimatedEmojiBuilder<FA, FB> {
    pub fn max_frequency(self, max_frequency: u32) -> AnimatedEmojiBuilder<MaxFrequencySet, FB> {
        assert!(max_frequency > 0);
        AnimatedEmojiBuilder {
            max_frequency: MaxFrequencySet(max_frequency),
            frames: self.frames,
        }
    }
    pub fn frames(self, frames: &'a [char]) -> AnimatedEmojiBuilder<FA, FramesSet<'a>> {
        assert!(!frames.is_empty());
        AnimatedEmojiBuilder {
            max_frequency: self.max_frequency,
            frames: FramesSet(frames),
        }
    }
}

impl<'a> AnimatedEmojiBuilder<MaxFrequencySet, FramesSet<'a>> {
    pub fn build(self) -> AnimatedEmoji<'a> {
        let max_frequency = self.max_frequency.0;
        let frames = self.frames.0;
        AnimatedEmoji::new(max_frequency, SystemTime::UNIX_EPOCH, frames)
    }
}
