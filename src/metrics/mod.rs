pub mod bluetooth;
pub mod cpu;
pub mod date;
pub mod mem;
pub mod net;
pub mod update;
pub mod xkblayout;
pub mod battery;

pub use battery::BatteryMetric;
pub use bluetooth::BluetoothChargeMetric;
pub use cpu::CpuMetric;
pub use date::DateMetric;
pub use mem::MemMetric;
pub use net::NetMetric;
pub use update::UpdatesMetric;
pub use xkblayout::XkbLayoutMetric;
