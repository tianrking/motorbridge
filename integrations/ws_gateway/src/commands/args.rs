use crate::model::{ServerConfig, Target, Transport, Vendor};

use super::common::parse_hex_or_dec;

pub(crate) fn parse_args() -> Result<ServerConfig, String> {
    let mut bind = "127.0.0.1:9002".to_string();
    let mut vendor = Vendor::Damiao;
    let mut transport = Transport::Auto;
    let mut channel = "can0".to_string();
    let mut serial_port = "/dev/ttyACM0".to_string();
    let mut serial_baud = 921600u32;
    let mut model = "auto".to_string();
    let mut motor_id = 0x01u16;
    let mut feedback_id = 0x11u16;
    let mut dt_ms = 20u64;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0usize;
    while i < args.len() {
        let k = &args[i];
        if k == "--help" || k == "-h" {
            println!(
                "ws_gateway\n\
Usage (router mode, recommended):\n\
  cargo run -p ws_gateway --release -- --bind 127.0.0.1:9002\n\
\n\
Optional defaults (only used when WS message omits target fields):\n\
  --vendor damiao|robstride|hexfellow|myactuator|hightorque\n\
  --transport auto|socketcan|socketcanfd|dm-serial\n\
  --channel can0 --serial-port /dev/ttyACM0 --serial-baud 921600\n\
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20\n"
            );
            std::process::exit(0);
        }
        let next = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {k}"))?;
        match k.as_str() {
            "--bind" => bind = next.clone(),
            "--vendor" => vendor = Vendor::from_str(next)?,
            "--transport" => transport = Transport::from_str(next)?,
            "--channel" => channel = next.clone(),
            "--serial-port" => serial_port = next.clone(),
            "--serial-baud" => {
                serial_baud = next
                    .parse::<u32>()
                    .map_err(|e| format!("invalid --serial-baud: {e}"))?;
            }
            "--model" => model = next.clone(),
            "--motor-id" => motor_id = parse_hex_or_dec(next)?,
            "--feedback-id" => feedback_id = parse_hex_or_dec(next)?,
            "--dt-ms" => {
                dt_ms = next
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --dt-ms: {e}"))?;
            }
            _ => return Err(format!("unknown arg: {k}")),
        }
        i += 2;
    }

    if vendor == Vendor::Robstride {
        if model == "4340P" || model == "4340" {
            model = "rs-00".to_string();
        }
        if feedback_id == 0x11 {
            feedback_id = 0xFF;
        }
    } else if vendor == Vendor::Myactuator {
        if model == "4340P" || model == "4340" {
            model = "X8".to_string();
        }
        if feedback_id == 0x11 {
            feedback_id = 0x241;
        }
    } else if vendor == Vendor::Hexfellow {
        if model == "4340P" || model == "4340" {
            model = "hexfellow".to_string();
        }
        if feedback_id == 0x11 {
            feedback_id = 0x00;
        }
    } else if vendor == Vendor::Hightorque {
        if model == "4340P" || model == "4340" {
            model = "hightorque".to_string();
        }
        if feedback_id == 0x11 {
            feedback_id = 0x01;
        }
    }

    Ok(ServerConfig {
        bind,
        target: Target {
            vendor,
            transport,
            channel,
            serial_port,
            serial_baud,
            model,
            motor_id,
            feedback_id,
        },
        dt_ms,
    })
}
