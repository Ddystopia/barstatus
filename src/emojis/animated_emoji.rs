use super::animated_emoji_builder::{AnimatedEmojiBuilder, FramesNotSet, MaxFrequencyNotSet};
use crate::duration_since;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnimatedEmoji<'a> {
    max_frequency: u32,
    frame: usize,
    previous_frame_update: SystemTime,
    frames: &'a [char],
}

impl<'a> AnimatedEmoji<'a> {
    pub(super) fn new(
        max_frequency: u32,
        previous_frame_update: SystemTime,
        frames: &'a [char],
    ) -> AnimatedEmoji<'a> {
        AnimatedEmoji {
            max_frequency,
            frame: 0,
            previous_frame_update,
            frames,
        }
    }

    pub fn builder<'b>() -> AnimatedEmojiBuilder<'b, MaxFrequencyNotSet, FramesNotSet> {
        AnimatedEmojiBuilder::<MaxFrequencyNotSet, FramesNotSet>::default()
    }
    /// speed is a value between 0 and 1
    pub fn get_frame(&mut self, speed: f32) -> char {
        assert!((0.0..=1.0).contains(&speed));
        let frequency = speed * self.max_frequency as f32;
        let fps = self.frames.len() as f32 * frequency;
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
