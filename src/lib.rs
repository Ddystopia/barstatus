use std::{fmt::Display, time::Duration};

pub(crate) mod read_line;

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

pub trait Metric: Display + std::fmt::Debug {
    fn update(&mut self) {}
    fn timeout(&self) -> Duration;
}

#[macro_export]
macro_rules! generic_for_each {
    ($list:ident, |$x:ident: &mut impl $trait:ident $( + $trait_rest:ident )*| $body:expr) => {
        generic_for_each!($list, (), |$x: &mut impl $trait $( + $trait_rest)*, _tmp: ()| $body);
    };
    ($list:ident, $deps:expr, |$x:ident: &mut impl $trait:ident $( + $trait_rest:ident )*, $deps_var:ident:$dep_ty:ty| $body:expr) => {
        {
            use frunk::hlist::{HCons, HNil};
            #[allow(non_camel_case_types)]
            trait $list {
                fn for_each(&mut self, dep: $dep_ty);
            }
            impl<H: $trait $(+ $trait_rest)*, T: $list> $list for HCons<H, T> {
                #[inline(always)]
                fn for_each(&mut self, dep: $dep_ty) {
                    #[allow(unused_parens)]
                    #[inline(always)]
                    fn generic_call($x: &mut (impl $trait $(+ $trait_rest)*), $deps_var: $dep_ty) {
                        _ = { $body }
                    }
                    generic_call(&mut self.head, dep);
                    self.tail.for_each(dep);
                }
            }

            impl $list for HNil {
                #[inline(always)]
                fn for_each(&mut self, dep: $dep_ty) {}
            }

            $list.for_each($deps);
        }
    };
}
