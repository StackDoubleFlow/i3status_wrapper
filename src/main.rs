use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(Deserialize, Serialize, Debug)]
struct Block {
    name: String,
    markup: String,
    full_text: String,
}

fn headset_volume() -> Result<String> {
    let output = Command::new("headsetcontrol").args(["-c", "-b"]).output()?;
    Ok(String::from_utf8(output.stdout)?)
}

fn main() -> Result<()> {
    let status_command = Command::new("i3status").stdout(Stdio::piped()).spawn()?;
    let status_stdout = status_command.stdout.context("i3status has no stdout")?;
    let mut status_lines = BufReader::new(status_stdout).lines();

    let header = status_lines.next().context("no header")??;
    println!("{}", header);
    println!("[");

    for blocks in status_lines.skip(1) {
        let blocks = blocks?;
        let blocks = blocks.trim_start_matches(',');
        let mut blocks: Vec<Block> = serde_json::from_str(blocks)?;

        let headset_volume = headset_volume()?;
        let headset_volume = if headset_volume == "-1" {
            "Charging".to_string()
        } else {
            format!("{}%", headset_volume)
        };
        blocks.insert(
            0,
            Block {
                name: "headset_volume".to_string(),
                markup: "pango".to_string(),
                full_text: format!(
                    "<span background='#007e91'> \u{F7CD} {} </span>",
                    headset_volume
                ),
            },
        );

        println!("{},", serde_json::to_string(&blocks)?);
    }

    Ok(())
}
