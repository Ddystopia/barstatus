use std::{
    io::{Cursor, Write},
    thread,
    time::Duration,
};

use anyhow::Context;
use barstatus::{
    generic_for_each,
    metrics::{
        BatteryMetric, BluetoothChargeMetric, CpuMetric, DateMetric, NetMetric, UpdatesMetric,
        XkbLayoutMetric,
    },
    Metric,
};
use frunk::hlist;
use future_to_stream::AnyStream;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

mod future_to_stream;
mod xsetroot;

const FPS: f64 = 71.;
const LOOP_TIME: Duration = Duration::from_nanos((1_000_000_000. / FPS) as u64);

macro_rules! merge {
    [$stream:expr] => {
        $stream
    };
    [$stream:expr, $($streams:expr),+] => {
        $stream.merge(merge![$($streams),+])
    };
}

fn main2(metric1: impl barstatus::Metric2, metric2: impl barstatus::Metric2) -> anyhow::Result<()> {
    env_logger::init();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Unable to create tokio runtime")?;

    rt.block_on(async {
        let mut interval = tokio::time::interval(LOOP_TIME);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let interval = IntervalStream::new(interval);
        let metric1_future = metric1.future().to_any_stream();
        let metric2_future = metric2.future().to_any_stream();

        let stream = merge![metric1_future, metric2_future, interval];
        let mut stream = std::pin::pin!(stream);

        while let Some(time) = stream.next().await {
            eprintln!("{:?}", time);

            let mut buf: [u8; 1024] = [0; 1024];
            let mut writer = Cursor::new(&mut buf[..]);

            write!(writer, "{}", metric1.display()).unwrap();
            write!(writer, " | ").unwrap();
            write!(writer, "{}", metric2.display()).unwrap();
        }
    });

    Ok(())
}

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
            &mut writer,
            |metric: &mut impl Metric, writer: &mut Cursor<&mut [u8]>| {
                println!("{:?}", metric);
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
        let position = writer.position() as usize + 1;
        let line = std::ffi::CStr::from_bytes_with_nul(&buf[..position]).unwrap();

        if let Err(e) = xsetroot::set_on_bar(line) {
            eprintln!("Error while setting on bar: {e}");
            break;
        };

        generic_for_each!(metrics, |metric: &mut impl Metric| metric.update());

        thread::sleep(LOOP_TIME.saturating_sub(loop_start.elapsed()));
    }
}
