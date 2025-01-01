#![cfg_attr(not(feature = "xsetroot_dyn"), forbid(unsafe_code))]
#![feature(never_type)]

// todo: adaptive frame rate. We do not need 71 fps all the time, only when
//       cat animation is running at full speed. Performance is good even at 71
//       fps, but adaptation will make this program nearly transparent.

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
    [$stream:expr $(, $streams:expr)+ $(,)?] => {
        $stream.merge(merge![$($streams),+])
    };
}

/// "Spawns" a loop that updates a metric every `interval` duration.
/// Note that it give a stream yielding any `T` - this is because it never
/// actually yields, so we can say we yield any `T`.
fn update_metric_in_interval<'a, M: Metric, T>(
    name: &'static str, // note: maybe use `Metric::name`
    interval: Duration,
    metric: &'a M,
) -> impl tokio_stream::Stream<Item = T> + use<'a, M, T> {
    FutureToStream::new(async move {
        let mut interval = tokio::time::interval(interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if let Err(err) = metric.update().await {
                log::error!("Error in {name}: {err}");
            }
        }
    })
}

fn main() {
    env_logger::init();

    let net_metric = NetMetric::default();
    let cpu_metric = CpuMetric::default();
    // bluetoothctl, grep, sed
    let bluetooth_metric = BluetoothChargeMetric::default();
    // xkb-switch
    let xkb_metric = XkbLayoutMetric::default();
    // checkupdates
    let updates_metric = UpdatesMetric::default();
    let battery_metric = BatteryMetric::new(80);
    let date_metric = DateMetric::default();

    let main = async move {
        let mut frame_interval = tokio::time::interval(LOOP_TIME);
        frame_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let frame_interval = IntervalStream::new(frame_interval);

        // Those are being polled together with `frame_interval`, but
        // never yield any value. In this particular case `LocalSet` might be
        // an option too, but this approach allows not only `!Send`, but also
        // not `'static` futures, making it very appealing to use.
        let background_metrics = merge![
            update_metric_in_interval("Net", Duration::from_secs(2), &net_metric),
            update_metric_in_interval("Cpu", Duration::from_millis(600), &cpu_metric),
            update_metric_in_interval("Bluetooth", Duration::from_secs(5), &bluetooth_metric),
            update_metric_in_interval("Xkb", Duration::from_millis(300), &xkb_metric),
            update_metric_in_interval("Updates", Duration::from_secs(60), &updates_metric),
            update_metric_in_interval("Battery", Duration::from_secs(1), &battery_metric),
        ];

        // So only `frame_interval` yieods, others are just being continuously
        // polled
        let mut stream = pin!(background_metrics.merge(frame_interval));

        while let Some(_time) = stream.next().await {
            let mut buf: [u8; 256] = [0; 256];
            let mut writer = Cursor::new(&mut buf[..]);

            let mut metrics = hlist![
                &net_metric,
                &cpu_metric,
                &bluetooth_metric,
                &xkb_metric,
                &updates_metric,
                &battery_metric,
                &date_metric
            ];

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
