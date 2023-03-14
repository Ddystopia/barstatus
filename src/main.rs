use barstatus::{
  metrics::{BatteryMetric, CPUMetric, DateMetric, NetMetric, UpdatesMetric, XkbLayoutMetric},
  Metric,
};
use std::time::Duration;
use std::{process::ExitStatus, thread};

use std::process::Command;

const LOOP_TIME: Duration = Duration::from_millis(30);

fn main() {
  let mut metrics: Vec<Box<dyn Metric>> = vec![
    Box::new(NetMetric::new(Duration::from_secs(2))),
    Box::new(CPUMetric::new(Duration::from_millis(600))),
    // Box::new(BluetoothChargeMetric::new()),
    Box::new(XkbLayoutMetric::new(Duration::from_millis(200))),
    Box::new(UpdatesMetric::new(Duration::from_secs(60))),
    Box::new(BatteryMetric::new(80)),
    Box::new(DateMetric::new()),
  ];

  loop {
    let val = metrics
      .iter()
      .map(|m| m.get_value())
      .filter(|s| !s.is_empty())
      .collect::<Vec<_>>()
      .join(" | ");

    match set_on_bar(&format!("{: >93}", val)) {
      Ok(_) => (),
      Err(e) => {
        eprintln!("Error while setting on bar: {}", e);
        break;
      }
    };

    metrics.iter_mut().for_each(|m| m.update());

    thread::sleep(LOOP_TIME);
  }
}

fn set_on_bar(val: &str) -> Result<ExitStatus, std::io::Error> {
  Command::new("xsetroot")
    .args(["-name", val])
    .spawn()?
    .wait()
}
