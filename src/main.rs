use std::{
    io::{Cursor, Write},
    process::{Command, ExitStatus},
    thread,
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

const FPS: f64 = 71.;
const LOOP_TIME: Duration = Duration::from_nanos((1_000_000_000. / FPS) as u64);

fn main() {
    env_logger::init();

    let mut metrics = hlist![
        NetMetric::new(Duration::from_secs(2)),
        CpuMetric::new(Duration::from_millis(600)),
        BluetoothChargeMetric::new(),
        XkbLayoutMetric::new(Duration::from_millis(300)), // fixme: slow
        UpdatesMetric::new(Duration::from_secs(60)),
        BatteryMetric::new(80),
        DateMetric::new(),
    ];

    // let mut last = std::time::Instant::now();
    loop {
        // eprintln!("{:?} vs {LOOP_TIME:?}", last.elapsed());
        // last = std::time::Instant::now();
        let loop_start = std::time::Instant::now();
        let mut buf: [u8; 1024] = [0; 1024];
        let mut writer = Cursor::new(&mut buf[..]);

        generic_for_each!(
            metrics,
            Metric,
            &mut writer, //
            |metric, writer: &mut Cursor<&mut [u8]>| {
                let prev_pos = writer.position();

                if write!(writer, "{metric}").is_err() {
                    log::error!("Error while writing metric {metric:?}");
                }

                if prev_pos != writer.position() {
                    write!(writer, " | ").unwrap();
                }
            }
        );

        let position = writer.position() as usize;
        let line = std::str::from_utf8(&buf[..position]).unwrap();
        let line = line.trim_end_matches(" | ");

        let mut buf: [u8; 1024] = [0; 1024];
        let mut writer = std::io::Cursor::new(&mut buf[..]);
        write!(writer, "{line: >93}").unwrap();
        let position = writer.position() as usize;
        let line = std::str::from_utf8(&buf[..position]).unwrap();

        if let Err(e) = set_on_bar(line) {
            eprintln!("Error while setting on bar: {e}");
            break;
        };

        generic_for_each!(metrics, Metric, |metric| metric.update());

        thread::sleep(LOOP_TIME.saturating_sub(loop_start.elapsed()));
    }
}

fn set_on_bar(val: &str) -> Result<ExitStatus, std::io::Error> {
    // fixme: slow
    Command::new("xsetroot")
        .args(["-name", val])
        .spawn()?
        .wait()
}
