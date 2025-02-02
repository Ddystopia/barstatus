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
        AnimatedEmoji { max_frequency, min_frequency, frame: 0, previous_frame_update, frames }
    }

    #[must_use]
    pub fn builder() -> AnimatedEmojiBuilder<MaxFrequencyNotSet, FramesNotSet> {
        AnimatedEmojiBuilder::default()
    }
    /// # Panics
    /// if speed is not a value between 0 and 1
    pub fn next_frame(&mut self, speed: f64) -> char {
        assert!((0.0..=1.0).contains(&speed), "Speed must be a value between 0 and 1");
        let frequency = self.min_frequency + speed * (self.max_frequency - self.min_frequency);
        let fps = self.frames.len() as f64 * frequency;
        let elapsed = self.previous_frame_update.map(|it| it.elapsed());

        let frames_to_skip = elapsed.map_or(1., |it| it.as_secs_f64() * fps);
        let frames_to_skip = frames_to_skip.floor() as usize;

        if frames_to_skip > 0 {
            self.frame += frames_to_skip;
            self.frame %= self.frames.len();
            self.previous_frame_update = Some(Instant::now());
        }

        self.frames[self.frame]
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.previous_frame_update = None;
    }
}
