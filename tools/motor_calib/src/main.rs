use motor_vendor_damiao::DamiaoController;
use std::collections::HashMap;
use std::time::Duration;

fn parse_opts(raw: &[String]) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut it = raw.iter().peekable();
    while let Some(k) = it.next() {
        if !k.starts_with("--") {
            continue;
        }
        let key = k.trim_start_matches("--").to_string();
        match it.peek() {
            Some(v) if !v.starts_with("--") => {
                if let Some(val) = it.next() {
                    out.insert(key, val.to_string());
                }
            }
            _ => {
                out.insert(key, "1".to_string());
            }
        }
    }
    out
}

fn get_str(opts: &HashMap<String, String>, key: &str, default: &str) -> String {
    opts.get(key)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

fn get_u64(opts: &HashMap<String, String>, key: &str, default: u64) -> Result<u64, String> {
    match opts.get(key) {
        Some(v) => v
            .parse::<u64>()
            .map_err(|e| format!("invalid --{key}: {e}")),
        None => Ok(default),
    }
}

fn parse_u16_hex_or_dec(v: &str, key: &str) -> Result<u16, String> {
    if let Some(hex) = v.strip_prefix("0x") {
        u16::from_str_radix(hex, 16).map_err(|e| format!("invalid --{key}: {e}"))
    } else {
        v.parse::<u16>()
            .map_err(|e| format!("invalid --{key}: {e}"))
    }
}

fn get_u16_hex_or_dec(
    opts: &HashMap<String, String>,
    key: &str,
    default: u16,
) -> Result<u16, String> {
    match opts.get(key) {
        Some(v) => parse_u16_hex_or_dec(v, key),
        None => Ok(default),
    }
}

fn get_opt_u16_hex_or_dec(
    opts: &HashMap<String, String>,
    key: &str,
) -> Result<Option<u16>, String> {
    match opts.get(key) {
        Some(v) => Ok(Some(parse_u16_hex_or_dec(v, key)?)),
        None => Ok(None),
    }
}

fn print_help() {
    println!(
        "motor_calib\n\
Usage:\n\
  cargo run -p motor_calib -- <command> [options]\n\n\
Commands:\n\
  scan      scan active motor IDs\n\
  set-id    set ESC_ID/MST_ID for one motor\n\
  verify    verify ESC_ID/MST_ID by reading rid=8/7\n\n\
Examples:\n\
  cargo run -p motor_calib -- scan --channel can0 --model 4310 --start-id 0x01 --end-id 0x10\n\
  cargo run -p motor_calib -- set-id --channel can0 --model 4310 \\\n    --motor-id 0x02 --feedback-id 0x12 --new-motor-id 0x05 --new-feedback-id 0x15 --store 1 --verify 1\n\
  cargo run -p motor_calib -- verify --channel can0 --model 4310 --motor-id 0x05 --feedback-id 0x15\n"
    );
}

fn print_scan_help() {
    println!(
        "motor_calib scan\n\
Options:\n\
  --channel can0\n\
  --model 4340\n\
  --start-id 0x01\n\
  --end-id 0x10\n\
  --feedback-base 0x10\n\
  --timeout-ms 80\n"
    );
}

fn print_set_id_help() {
    println!(
        "motor_calib set-id\n\
Options:\n\
  --channel can0\n\
  --model 4340\n\
  --motor-id 0x01\n\
  --feedback-id 0x11\n\
  --new-motor-id <id>\n\
  --new-feedback-id <id>\n\
  --store 1|0 (default 1)\n\
  --verify 1|0 (default 1)\n\
  --timeout-ms 1000\n"
    );
}

fn print_verify_help() {
    println!(
        "motor_calib verify\n\
Options:\n\
  --channel can0\n\
  --model 4340\n\
  --motor-id 0x01\n\
  --feedback-id 0x11\n\
  --timeout-ms 1000\n"
    );
}

fn cmd_scan(opts: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    if opts.contains_key("help") || opts.contains_key("h") {
        print_scan_help();
        return Ok(());
    }
    let channel = get_str(opts, "channel", "can0");
    let model = get_str(opts, "model", "4340");
    let start_id = get_u16_hex_or_dec(opts, "start-id", 0x01)?;
    let end_id = get_u16_hex_or_dec(opts, "end-id", 0x10)?;
    let feedback_base = get_u16_hex_or_dec(opts, "feedback-base", 0x10)?;
    let timeout_ms = get_u64(opts, "timeout-ms", 80)?;

    if end_id < start_id {
        return Err("end-id must be >= start-id".into());
    }

    println!(
        "[scan] channel={} model={} id_range=[0x{:X},0x{:X}] timeout_ms={}",
        channel, model, start_id, end_id, timeout_ms
    );

    let mut hits = Vec::new();
    for mid in start_id..=end_id {
        let fid = feedback_base + (mid & 0x0F);
        let controller = DamiaoController::new_socketcan(&channel)?;
        let motor = match controller.add_motor(mid, fid, &model) {
            Ok(m) => m,
            Err(_) => {
                let _ = controller.close_bus();
                continue;
            }
        };

        let esc = motor.get_register_u32(8, Duration::from_millis(timeout_ms));
        let mst = motor.get_register_u32(7, Duration::from_millis(timeout_ms));
        match (esc, mst) {
            (Ok(esc_id), Ok(mst_id)) => {
                println!(
                    "[hit] probe=0x{:02X} esc_id=0x{:X} mst_id=0x{:X} feedback_probe=0x{:X}",
                    mid, esc_id, mst_id, fid
                );
                hits.push((mid, esc_id, mst_id));
            }
            _ => {
                println!("[.. ] probe=0x{:02X} no reply", mid);
            }
        }
        let _ = controller.close_bus();
    }

    println!("[scan] done: {} motor(s) found", hits.len());
    Ok(())
}

fn cmd_verify(opts: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    if opts.contains_key("help") || opts.contains_key("h") {
        print_verify_help();
        return Ok(());
    }
    let channel = get_str(opts, "channel", "can0");
    let model = get_str(opts, "model", "4340");
    let motor_id = get_u16_hex_or_dec(opts, "motor-id", 0x01)?;
    let feedback_id = get_u16_hex_or_dec(opts, "feedback-id", 0x11)?;
    let timeout_ms = get_u64(opts, "timeout-ms", 1000)?;

    println!(
        "[verify] channel={} model={} motor_id=0x{:X} feedback_id=0x{:X}",
        channel, model, motor_id, feedback_id
    );

    let controller = DamiaoController::new_socketcan(&channel)?;
    let motor = controller.add_motor(motor_id, feedback_id, &model)?;
    let esc = motor.get_register_u32(8, Duration::from_millis(timeout_ms))?;
    let mst = motor.get_register_u32(7, Duration::from_millis(timeout_ms))?;
    println!("[verify] rid=8 (ESC_ID)=0x{:X}", esc);
    println!("[verify] rid=7 (MST_ID)=0x{:X}", mst);
    controller.close_bus()?;
    Ok(())
}

fn cmd_set_id(opts: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    if opts.contains_key("help") || opts.contains_key("h") {
        print_set_id_help();
        return Ok(());
    }
    let channel = get_str(opts, "channel", "can0");
    let model = get_str(opts, "model", "4340");
    let motor_id = get_u16_hex_or_dec(opts, "motor-id", 0x01)?;
    let feedback_id = get_u16_hex_or_dec(opts, "feedback-id", 0x11)?;
    let new_motor_id = get_opt_u16_hex_or_dec(opts, "new-motor-id")?.unwrap_or(motor_id);
    let new_feedback_id = get_opt_u16_hex_or_dec(opts, "new-feedback-id")?.unwrap_or(feedback_id);
    let store = get_u64(opts, "store", 1)? != 0;
    let verify = get_u64(opts, "verify", 1)? != 0;
    let timeout_ms = get_u64(opts, "timeout-ms", 1000)?;

    println!(
        "[set-id] old motor_id=0x{:X} feedback_id=0x{:X} -> new motor_id=0x{:X} feedback_id=0x{:X}",
        motor_id, feedback_id, new_motor_id, new_feedback_id
    );

    let controller = DamiaoController::new_socketcan(&channel)?;
    let motor = controller.add_motor(motor_id, feedback_id, &model)?;

    // Important order for robust updates: MST_ID first, then ESC_ID.
    if new_feedback_id != feedback_id {
        motor.write_register_u32(7, new_feedback_id as u32)?;
        println!("[set-id] write rid=7 (MST_ID)=0x{:X}", new_feedback_id);
    }
    if new_motor_id != motor_id {
        motor.write_register_u32(8, new_motor_id as u32)?;
        println!("[set-id] write rid=8 (ESC_ID)=0x{:X}", new_motor_id);
    }
    if store {
        motor.store_parameters()?;
        println!("[set-id] store parameters sent");
    }
    controller.close_bus()?;

    if verify {
        std::thread::sleep(Duration::from_millis(120));
        let verify_ctrl = DamiaoController::new_socketcan(&channel)?;
        let verify_motor = verify_ctrl.add_motor(new_motor_id, new_feedback_id, &model)?;
        let esc = verify_motor.get_register_u32(8, Duration::from_millis(timeout_ms))?;
        let mst = verify_motor.get_register_u32(7, Duration::from_millis(timeout_ms))?;
        println!("[set-id] verify rid=8 (ESC_ID)=0x{:X}", esc);
        println!("[set-id] verify rid=7 (MST_ID)=0x{:X}", mst);
        verify_ctrl.close_bus()?;

        if esc != new_motor_id as u32 || mst != new_feedback_id as u32 {
            return Err(format!(
                "verify failed: expected ESC_ID=0x{:X}, MST_ID=0x{:X}, got ESC_ID=0x{:X}, MST_ID=0x{:X}",
                new_motor_id, new_feedback_id, esc, mst
            )
            .into());
        }
        println!("[set-id] verify ok");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.is_empty() || raw[0] == "--help" || raw[0] == "-h" {
        print_help();
        return Ok(());
    }

    let cmd = raw[0].as_str();
    let opts = parse_opts(&raw[1..]);

    match cmd {
        "scan" => cmd_scan(&opts),
        "set-id" => cmd_set_id(&opts),
        "verify" => cmd_verify(&opts),
        _ => {
            print_help();
            Err(format!("unknown command: {cmd}").into())
        }
    }
}
