mod args;
mod damiao_cli;
mod hightorque_cli;
mod myactuator_cli;
mod robstride_cli;

use args::{get_str, get_u16_hex_or_dec, print_help};
use damiao_cli::run_damiao;
use hightorque_cli::run_hightorque;
use myactuator_cli::run_myactuator;
use robstride_cli::run_robstride;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::parse_args();
    if args.is_empty() || args.contains_key("help") {
        print_help();
        return Ok(());
    }

    let vendor = get_str(&args, "vendor", "damiao");
    let channel = get_str(&args, "channel", "can0");
    let default_model = if vendor == "robstride" {
        "rs-00"
    } else if vendor == "hightorque" {
        "hightorque"
    } else if vendor == "myactuator" {
        "X8"
    } else {
        "4340"
    };
    let model = get_str(&args, "model", default_model);
    let motor_id = get_u16_hex_or_dec(&args, "motor-id", 0x01)?;
    let feedback_default = if vendor == "robstride" {
        0x00FF
    } else if vendor == "hightorque" {
        0x0001
    } else if vendor == "myactuator" {
        0x0241
    } else {
        0x0011
    };
    let feedback_id = get_u16_hex_or_dec(&args, "feedback-id", feedback_default)?;
    let mode = get_str(
        &args,
        "mode",
        if vendor == "robstride" {
            "ping"
        } else if vendor == "hightorque" {
            "read"
        } else if vendor == "myactuator" {
            "status"
        } else if vendor == "all" {
            "scan"
        } else {
            "mit"
        },
    );

    println!(
        "vendor={} channel={} model={} motor_id=0x{:X} feedback_id=0x{:X} mode={}",
        vendor, channel, model, motor_id, feedback_id, mode
    );

    if vendor == "all" {
        if mode != "scan" {
            return Err("vendor=all currently supports --mode scan only".into());
        }
        let damiao_model = get_str(&args, "damiao-model", "4340P");
        let robstride_model = get_str(&args, "robstride-model", "rs-00");
        let hightorque_model = get_str(&args, "hightorque-model", "hightorque");
        let myactuator_model = get_str(&args, "myactuator-model", "X8");
        println!(
            "[scan-all] running Damiao scan with model_hint={}, RobStride scan with model_hint={}, HighTorque scan(by ht_can) with model_hint={}, then MyActuator scan with model_hint={}",
            damiao_model, robstride_model, hightorque_model, myactuator_model
        );
        run_damiao(&args, &channel, &damiao_model, motor_id, 0x0011)?;
        run_robstride(
            &args,
            &channel,
            &robstride_model,
            motor_id,
            0x00FF,
            "robstride",
        )?;
        let mut ht_args = args.clone();
        ht_args.insert("mode".to_string(), "scan".to_string());
        if !ht_args.contains_key("start-id") {
            ht_args.insert("start-id".to_string(), "1".to_string());
        }
        if !ht_args.contains_key("end-id") {
            ht_args.insert("end-id".to_string(), "32".to_string());
        }
        let _ = hightorque_model;
        run_hightorque(&ht_args, &channel, motor_id)?;
        run_myactuator(&args, &channel, &myactuator_model, motor_id, 0x0241)?;
        return Ok(());
    }

    match vendor.as_str() {
        "damiao" => run_damiao(&args, &channel, &model, motor_id, feedback_id),
        "robstride" => run_robstride(&args, &channel, &model, motor_id, feedback_id, "robstride"),
        "hightorque" => run_hightorque(&args, &channel, motor_id),
        "myactuator" => run_myactuator(&args, &channel, &model, motor_id, feedback_id),
        _ => Err(format!("unknown vendor: {vendor}").into()),
    }
}
