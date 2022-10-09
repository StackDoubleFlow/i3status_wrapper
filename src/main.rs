use anyhow::{Context, Result};
use libg933::battery::{BatteryStatus, ChargingStatus};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static BATTERY_STATUS: Mutex<Option<BatteryStatus>> = Mutex::new(None);

#[derive(Deserialize, Serialize, Debug)]
struct Block<'a> {
    name: &'a str,
    markup: &'a str,
    full_text: &'a str,
}

fn update_battery_status() {
    let mut devices = libg933::find_devices().unwrap();
    if devices.is_empty() {
        let mut lock = BATTERY_STATUS.lock().unwrap();
        *lock = None;
        return;
    }
    let device = devices.values_mut().next().unwrap();

    if let Ok(battery_status) = device.get_battery_status() {
        let mut lock = BATTERY_STATUS.lock().unwrap();
        *lock = Some(battery_status);
    }
}

fn headset_battery() -> Result<String> {
    let battery_status = BATTERY_STATUS.lock().unwrap();
    Ok(if let Some(battery_status) = &*battery_status {
        let mut status = format!("{:.01}%", battery_status.charge);
        if matches!(battery_status.charging_status, ChargingStatus::Charging(_)) {
            status.push_str(" [\u{f583}]")
        }
        status
    } else {
        "Disconnected".to_string()
    })
}

fn main() -> Result<()> {
    let status_command = Command::new("i3status").stdout(Stdio::piped()).spawn()?;
    let status_stdout = status_command.stdout.context("i3status has no stdout")?;
    let mut status_lines = BufReader::new(status_stdout).lines();

    let header = status_lines.next().context("no header")??;
    println!("{}", header);
    println!("[");

    thread::spawn(|| loop {
        update_battery_status();
        thread::sleep(Duration::from_secs(5))
    });

    for blocks in status_lines.skip(1) {
        let blocks = blocks?;
        let blocks = blocks.trim_start_matches(',');
        let mut blocks: Vec<Block> = serde_json::from_str(blocks)?;

        let headset_battery = headset_battery()?;
        let full_text = format!(
            "<span background='#007e91'> \u{F7CD} {} </span>",
            headset_battery
        );
        blocks.insert(
            0,
            Block {
                name: "headset_battery",
                markup: "pango",
                full_text: &full_text,
            },
        );

        println!("{},", serde_json::to_string(&blocks)?);
    }

    Ok(())
}
