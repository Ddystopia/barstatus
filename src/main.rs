#![cfg_attr(not(feature = "xsetroot_dyn"), forbid(unsafe_code))]
#![feature(never_type)]

use std::{
    io::{Cursor, Write},
    pin::pin,
    time::Duration,
};

use barstatus::{
    generic_for_each,
    metrics::{
        BatteryMetric, BluetoothChargeMetric, CpuMetric, DateMetric, NetMetric, UpdatesMetric,
        XkbLayoutMetric,
    },
    Metric,
};
use frunk::hlist;
use future_to_stream::FutureToStream;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

mod future_to_stream;
mod xsetroot;

const FPS: f64 = 71.;
const LOOP_TIME: Duration = Duration::from_nanos((1_000_000_000. / FPS) as u64);

/// `merge![a, b, c]` is equivalent to `a.merge(b.merge(c))`
macro_rules! merge {
    [$stream:expr] => {
        $stream
    };
    [$stream:expr, $($streams:expr),+] => {
        $stream.merge(merge![$($streams),+])
    };
}

macro_rules! start_metric {
    ($metrics:expr, $metric:ty) => {
        FutureToStream::new((*$metrics.get::<&$metric, _>()).start())
    };
}

fn main() {
    env_logger::init();

    let mut metrics = hlist![
        &NetMetric::new(Duration::from_secs(2)),
        &CpuMetric::new(Duration::from_millis(600)),
        // bluetoothctl, grep, sed
        &BluetoothChargeMetric::new(),
        // xkb-switch
        &XkbLayoutMetric::new(Duration::from_millis(300)),
        // checkupdates
        &UpdatesMetric::new(Duration::from_secs(60)),
        &BatteryMetric::new(80), // once a second takes 1ms
        &DateMetric::new(),
    ];

    let main = async move {
        let mut interval = tokio::time::interval(LOOP_TIME);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let interval = IntervalStream::new(interval);

        let mut stream = pin!(merge![
            start_metric!(metrics, NetMetric),
            start_metric!(metrics, CpuMetric),
            start_metric!(metrics, BluetoothChargeMetric),
            start_metric!(metrics, XkbLayoutMetric),
            start_metric!(metrics, UpdatesMetric),
            start_metric!(metrics, BatteryMetric),
            start_metric!(metrics, DateMetric),
            interval
        ]);

        while let Some(_time) = stream.next().await {
            let mut buf: [u8; 256] = [0; 256];
            let mut writer = Cursor::new(&mut buf[..]);

            generic_for_each!(
                metrics,
                &mut writer,
                |metric: &mut impl Metric, writer: &mut Cursor<&mut [u8]>| {
                    let prev_pos = writer.position();

                    if let Err(err) = write!(writer, "{}", metric.display()) {
                        log::error!("Error while writing metric {}: {err}", metric.name());
                    }

                    if prev_pos != writer.position() {
                        if let Err(err) = write!(writer, " | ") {
                            log::error!("Error while writing separator: {err}");
                        }
                    }
                }
            );

            let position = writer.position().min(93) as usize;
            let Ok(line) = std::str::from_utf8(&buf[..position]) else {
                log::error!("Error while converting to utf8 (maybe in the middle of the emoji)");
                continue;
            };
            let line = line.trim_end_matches(" | ");

            // Purposefully block the executor. There are no tasks except this one.
            if let Err(e) = xsetroot::set_on_bar(line) {
                log::error!("Error while setting on bar: {e}");
                break;
            };
        }

        unreachable!("Interval stream cannot end");
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");

    rt.block_on(main);
}
