use super::AnimatedEmoji;
use std::num::NonZeroU32;

// The states to represent whether each field is set
pub struct MaxFrequencySet(NonZeroU32);
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
    fn default() -> Self {
        Self {
            max_frequency: MaxFrequencyNotSet,
            frames: FramesNotSet,
        }
    }
}

impl<'a, FA, FB> AnimatedEmojiBuilder<FA, FB> {
    /// Sets the maximum frequency of the animated emoji.
    ///
    /// # Arguments
    ///
    /// * `max_frequency` - A u32 that represents the maximum frequency that an emoji can be animated. This value must be greater than 0.
    ///
    /// # Returns
    ///
    /// This function returns an `AnimatedEmojiBuilder` with the `MaxFrequencySet` marker type, and the same `frames` marker type as the original builder.
    ///
    /// # Invariants
    ///
    /// * The `max_frequency` must be a positive integer. If it is not, this method will panic.
    ///
    #[inline]
    pub fn max_frequency(self, max_frequency: NonZeroU32) -> AnimatedEmojiBuilder<MaxFrequencySet, FB> {
        AnimatedEmojiBuilder {
            max_frequency: MaxFrequencySet(max_frequency),
            frames: self.frames,
        }
    }

    /// Sets the frames of the animated emoji.
    ///
    /// # Arguments
    ///
    /// * `frames` - A slice of characters that represent the frames of the animation.
    ///
    /// # Returns
    ///
    /// This function returns an `AnimatedEmojiBuilder` with the `FramesSet` marker type, and the same `max_frequency` marker type as the original builder.
    ///
    /// # Panics
    ///
    /// * The `frames` slice must not be empty. If it is, this method will panic.
    ///
    #[inline]
    pub fn frames(self, frames: &'a [char]) -> AnimatedEmojiBuilder<FA, FramesSet<'a>> {
        assert!(!frames.is_empty(), "The frames should not be empty");
        AnimatedEmojiBuilder {
            max_frequency: self.max_frequency,
            frames: FramesSet(frames),
        }
    }
}

impl<'a> AnimatedEmojiBuilder<MaxFrequencySet, FramesSet<'a>> {
    #[must_use]
    pub fn build(self) -> AnimatedEmoji<'a> {
        let max_frequency = self.max_frequency.0;
        let frames = self.frames.0;
        AnimatedEmoji::new(max_frequency, None, frames)
    }
}
