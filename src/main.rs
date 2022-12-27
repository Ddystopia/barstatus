use barstatus::{
  BluetoothChargeMetric, CPUMetric, DateMetric, Metric, NetMetric, UpdatesMetric, XkbLayoutMetric,
};
use chrono::DateTime;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

const LOOP_TIME: Duration = Duration::from_millis(200);

fn main() {
  let mut metrics: Vec<Box<dyn Metric>> = vec![
    Box::new(NetMetric::new(Duration::from_secs(2))),
    Box::new(CPUMetric::new(Duration::from_millis(600))),
    Box::new(BluetoothChargeMetric::new(Duration::from_secs(1))),
    Box::new(XkbLayoutMetric::new(Duration::ZERO)),
    Box::new(UpdatesMetric::new(Duration::from_secs(60 * 60))),
    Box::new(DateMetric::new(Duration::ZERO)),
  ];
  let mut timeouts = HashMap::new();
  let random_timestamp = "Wed, 18 Feb 2015 23:16:09 GMT";
  let start: SystemTime = DateTime::parse_from_rfc2822(random_timestamp)
    .unwrap()
    .into();
  for i in 0..metrics.len() {
    timeouts.insert(i, start);
  }

  loop {
    let val = metrics
      .iter()
      .map(|m| m.get_value())
      .filter(|s| s.len() > 0)
      .collect::<Vec<_>>()
      .join(" | ");

    let val = format!("{: >93}", val);

    Command::new("xsetroot")
      .args(["-name", &val])
      .spawn()
      .expect("xsetroot failed to execute");

    for i in 0..metrics.len() {
      let delta = SystemTime::now()
        .duration_since(timeouts[&i])
        .unwrap_or(Duration::ZERO);

      if delta > metrics[i].get_timeout() {
        metrics[i].update();
        timeouts.insert(i, SystemTime::now());
      }
    }

    thread::sleep(LOOP_TIME);
  }
}
