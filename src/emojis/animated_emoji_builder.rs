use super::AnimatedEmoji;

// The states to represent whether each field is set
pub struct MaxFrequencySet(f64);
pub struct MaxFrequencyNotSet;
pub struct FramesSet<'a>(&'a [char]);
pub struct FramesNotSet;

#[derive(Debug, Clone, PartialEq)]
pub struct AnimatedEmojiBuilder<MaxFrequencyState = MaxFrequencyNotSet, FramesState = FramesNotSet>
{
    max_frequency: MaxFrequencyState,
    min_frequency: f64,
    frames: FramesState,
}

impl Default for AnimatedEmojiBuilder<MaxFrequencyNotSet, FramesNotSet> {
    fn default() -> Self {
        Self { max_frequency: MaxFrequencyNotSet, frames: FramesNotSet, min_frequency: 0. }
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
    /// # Panics
    ///
    /// * The `max_frequency` must be positive. If it is not, this method will panic.
    ///
    #[inline]
    pub fn max_frequency(self, max_frequency: f64) -> AnimatedEmojiBuilder<MaxFrequencySet, FB> {
        assert!(max_frequency > 0., "The max frequency must be positive");
        AnimatedEmojiBuilder {
            max_frequency: MaxFrequencySet(max_frequency),
            min_frequency: self.min_frequency,
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
            min_frequency: self.min_frequency,
            frames: FramesSet(frames),
        }
    }

    #[inline]
    #[must_use]
    pub fn min_frequency(mut self, min_frequency: f64) -> Self {
        self.min_frequency = min_frequency;
        self
    }
}

impl<'a> AnimatedEmojiBuilder<MaxFrequencySet, FramesSet<'a>> {
    #[must_use]
    pub fn build(self) -> AnimatedEmoji<'a> {
        let max_frequency = self.max_frequency.0;
        let min_frequency = self.min_frequency;
        let frames = self.frames.0;
        AnimatedEmoji::new(max_frequency, min_frequency, None, frames)
    }
}
