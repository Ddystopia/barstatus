use barstatus::{
  metrics::{CPUMetric, DateMetric, NetMetric, UpdatesMetric, XkbLayoutMetric},
  Metric,
};
use std::thread;
use std::time::Duration;

use std::process::Command;

const LOOP_TIME: Duration = Duration::from_millis(30);

fn main() {
  let mut metrics: Vec<Box<dyn Metric>> = vec![
    Box::new(NetMetric::new(Duration::from_secs(2))),
    Box::new(CPUMetric::new(Duration::from_millis(600))),
    // Box::new(BluetoothChargeMetric::new()),
    Box::new(XkbLayoutMetric::new(Duration::from_millis(200))),
    Box::new(UpdatesMetric::new(Duration::from_secs(60))),
    Box::new(DateMetric::new()),
  ];
  
  loop {
    let val = metrics
      .iter()
      .map(|m| m.get_value())
      .filter(|s| !s.is_empty())
      .collect::<Vec<_>>()
      .join(" | ");

    set_on_bar(&format!("{: >93}", val));

    metrics.iter_mut().for_each(|m| m.update());

    thread::sleep(LOOP_TIME);
  }
}

fn set_on_bar(val: &str) {
  Command::new("xsetroot")
    .args(["-name", val])
    .spawn()
    .expect("xsetroot failed to execute")
    .wait()
    .expect("xsetroot failed to execute");
}
