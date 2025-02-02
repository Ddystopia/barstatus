#![cfg_attr(not(feature = "xsetroot_dyn"), forbid(unsafe_code))]
#![feature(never_type)]

// todo: adaptive frame rate. We do not need 71 fps all the time, only when
//       cat animation is running at full speed. Performance is good even at 71
//       fps, but adaptation will make this program nearly transparent.

use std::{
    io::{Cursor, Write},
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

mod xsetroot;

const FPS: f64 = 71.;
const LOOP_TIME: Duration = Duration::from_nanos((1_000_000_000. / FPS) as u64);

/// "Spawns" a loop that updates a metric every `interval` duration.
/// Note that it give a stream yielding any `T` - this is because it never
/// actually yields, so we can say we yield any `T`.
async fn metric_interval<'a, M: Metric>(
    name: &'static str, // note: maybe use `Metric::name`
    interval: Duration,
    metric: &'a M,
) {
    let mut interval = tokio::time::interval(interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        if let Err(err) = metric.update().await {
            log::error!("Error in {name}: {err}");
        }
    }
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

    let set = tokio::task::LocalSet::new();

    let mut frame_interval = tokio::time::interval(LOOP_TIME);
    frame_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let main = async {
        loop {
            frame_interval.tick().await;
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

    rt.block_on(set.run_until(async {
        tokio::join!(
            main,
            set.run_until(metric_interval("Net", Duration::from_secs(2), &net_metric)),
            set.run_until(metric_interval("Cpu", Duration::from_millis(600), &cpu_metric)),
            set.run_until(metric_interval("Bluetooth", Duration::from_secs(5), &bluetooth_metric)),
            set.run_until(metric_interval("Xkb", Duration::from_millis(300), &xkb_metric)),
            set.run_until(metric_interval("Updates", Duration::from_secs(60), &updates_metric)),
            set.run_until(metric_interval("Battery", Duration::from_secs(1), &battery_metric)),
        )
    }));
}
