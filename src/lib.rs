use std::time::Duration;
pub mod emojis {
    pub mod animated_emoji;
    pub mod animated_emoji_builder;

    pub use animated_emoji::AnimatedEmoji;
    pub use animated_emoji_builder::AnimatedEmojiBuilder;
}

pub mod metrics {
    pub mod battery;
    pub mod bluetooth;
    pub mod cpu;
    pub mod date;
    pub mod mem;
    pub mod net;
    pub mod update;
    pub mod xkblayout;

    pub use battery::BatteryMetric;
    pub use bluetooth::BluetoothChargeMetric;
    pub use cpu::CpuMetric;
    pub use date::DateMetric;
    pub use mem::MemMetric;
    pub use net::NetMetric;
    pub use update::UpdatesMetric;
    pub use xkblayout::XkbLayoutMetric;
}

pub trait Metric {
    fn update(&mut self) {}
    fn get_timeout(&self) -> Duration;
    fn get_value(&self) -> Option<String>;
}
