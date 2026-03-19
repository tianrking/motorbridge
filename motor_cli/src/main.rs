mod args;
mod damiao_cli;
mod robstride_cli;

use args::{get_str, get_u16_hex_or_dec, print_help};
use damiao_cli::run_damiao;
use robstride_cli::run_robstride;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::parse_args();
    if args.contains_key("help") {
        print_help();
        return Ok(());
    }

    let vendor = get_str(&args, "vendor", "damiao");
    let channel = get_str(&args, "channel", "can0");
    let default_model = if vendor == "robstride" {
        "rs-00"
    } else {
        "4340"
    };
    let model = get_str(&args, "model", default_model);
    let motor_id = get_u16_hex_or_dec(&args, "motor-id", 0x01)?;
    let feedback_default = if vendor == "robstride" {
        0x00FF
    } else {
        0x0011
    };
    let feedback_id = get_u16_hex_or_dec(&args, "feedback-id", feedback_default)?;
    let mode = get_str(
        &args,
        "mode",
        if vendor == "robstride" {
            "ping"
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
        println!(
            "[scan-all] running Damiao scan with model_hint={} then RobStride scan with model_hint={}",
            damiao_model, robstride_model
        );
        run_damiao(&args, &channel, &damiao_model, motor_id, 0x0011)?;
        run_robstride(&args, &channel, &robstride_model, motor_id, 0x00FF)?;
        return Ok(());
    }

    match vendor.as_str() {
        "damiao" => run_damiao(&args, &channel, &model, motor_id, feedback_id),
        "robstride" => run_robstride(&args, &channel, &model, motor_id, feedback_id),
        _ => Err(format!("unknown vendor: {vendor}").into()),
    }
}
