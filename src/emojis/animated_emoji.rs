use super::animated_emoji_builder::{AnimatedEmojiBuilder, FramesNotSet, MaxFrequencyNotSet};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimatedEmoji<'a> {
    max_frequency: f64,
    min_frequency: f64,
    frame: usize,
    previous_frame_update: Option<Instant>,
    frames: &'a [char],
}

impl<'a> AnimatedEmoji<'a> {
    pub(super) fn new(
        max_frequency: f64,
        min_frequency: f64,
        previous_frame_update: Option<Instant>,
        frames: &'a [char],
    ) -> AnimatedEmoji<'a> {
        AnimatedEmoji {
            max_frequency,
            min_frequency,
            frame: 0,
            previous_frame_update,
            frames,
        }
    }

    #[must_use]
    pub fn builder() -> AnimatedEmojiBuilder<MaxFrequencyNotSet, FramesNotSet> {
        AnimatedEmojiBuilder::default()
    }
    /// # Panics
    /// if speed is not a value between 0 and 1
    pub fn next_frame(&mut self, speed: f64) -> char {
        assert!(
            (0.0..=1.0).contains(&speed),
            "Speed must be a value between 0 and 1"
        );
        let frequency = self.min_frequency + speed * (self.max_frequency - self.min_frequency);
        let fps = self.frames.len() as f64 * frequency;
        let period_per_frame = Duration::from_millis((1000.0 / fps) as u64);

        let previous_update = self.previous_frame_update.map(|it| it.elapsed());

        if previous_update.map_or(true, |it| it > period_per_frame) {
            self.previous_frame_update = Some(Instant::now());
            self.frame += 1;
            self.frame %= self.frames.len();
        }

        self.frames[self.frame]
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.previous_frame_update = None;
    }
}
