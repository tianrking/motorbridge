use motor_vendor_damiao::{
    match_models_by_limits, model_limits as damiao_model_limits, suggest_models_by_limits,
    ControlMode as DamiaoControlMode, DamiaoController, DamiaoMotor,
};
use motor_vendor_robstride::{
    model_limits as robstride_model_limits, ControlMode as RobstrideControlMode, ParameterDataType,
    ParameterValue, RobstrideController,
};
use std::collections::HashMap;
use std::time::Duration;

fn parse_args() -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut it = std::env::args().skip(1).peekable();
    while let Some(k) = it.next() {
        if !k.starts_with("--") {
            continue;
        }
        if k == "--help" {
            out.insert("help".to_string(), "1".to_string());
            continue;
        }
        let key = k.trim_start_matches("--").to_string();
        match it.peek() {
            Some(v) if !v.starts_with("--") => {
                if let Some(val) = it.next() {
                    out.insert(key, val);
                }
            }
            _ => {
                out.insert(key, "1".to_string());
            }
        }
    }
    out
}

fn get_str(args: &HashMap<String, String>, key: &str, default: &str) -> String {
    args.get(key)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

fn get_f32(args: &HashMap<String, String>, key: &str, default: f32) -> Result<f32, String> {
    match args.get(key) {
        Some(v) => v
            .parse::<f32>()
            .map_err(|e| format!("invalid --{key}: {e}")),
        None => Ok(default),
    }
}

fn get_u64(args: &HashMap<String, String>, key: &str, default: u64) -> Result<u64, String> {
    match args.get(key) {
        Some(v) => v
            .parse::<u64>()
            .map_err(|e| format!("invalid --{key}: {e}")),
        None => Ok(default),
    }
}

fn parse_u16_hex_or_dec(s: &str, key: &str) -> Result<u16, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        u16::from_str_radix(hex, 16).map_err(|e| format!("invalid --{key}: {e}"))
    } else {
        s.parse::<u16>()
            .map_err(|e| format!("invalid --{key}: {e}"))
    }
}

fn get_u16_hex_or_dec(
    args: &HashMap<String, String>,
    key: &str,
    default: u16,
) -> Result<u16, String> {
    match args.get(key) {
        Some(v) => parse_u16_hex_or_dec(v, key),
        None => Ok(default),
    }
}

fn get_opt_u16_hex_or_dec(
    args: &HashMap<String, String>,
    key: &str,
) -> Result<Option<u16>, String> {
    match args.get(key) {
        Some(v) => Ok(Some(parse_u16_hex_or_dec(v, key)?)),
        None => Ok(None),
    }
}

fn print_help() {
    println!(
        "motor_cli\n\
Usage:\n\
  cargo run -p motor_cli --release -- \\\n\
    --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \\\n\
    --mode mit --pos 0 --vel 0 --kp 30 --kd 1 --tau 0 --loop 200 --dt-ms 20\n\n\
Vendors:\n\
  --vendor damiao    default\n\
  --vendor robstride\n\
  --vendor all       scan both vendors\n\n\
Damiao modes:\n\
  --mode scan | enable | disable | mit | pos-vel | vel | force-pos\n\n\
RobStride modes:\n\
  --mode ping | scan | enable | disable | mit | vel | read-param | write-param\n\n\
Common args:\n\
  --channel      default can0\n\
  --model        default depends on vendor (damiao=4340, robstride=rs-00)\n\
  --motor-id     default 0x01\n\
  --feedback-id  default 0x11 for Damiao, 0xFF for RobStride host-id compatibility\n\
  --loop         send cycles, default 1\n\
  --dt-ms        period ms, default 20\n\
  --ensure-mode  1/0, default 1\n\n\
Damiao extras:\n\
  --verify-model 1/0, default 1\n\
  --verify-timeout-ms  default 500\n\
  --verify-tol   default 0.2\n\
  --set-motor-id <id> --set-feedback-id <id> --store 1/0 --verify-id 1/0\n\n\
RobStride extras:\n\
  --param-id <hex|dec>      for read-param / write-param\n\
  --param-value <number>    for write-param\n\
  --start-id <hex|dec>      for scan, default 1\n\
  --end-id <hex|dec>        for scan, default 255\n\
  (scan auto-fallbacks to blind pulse probing if no ping replies)\n\
\n\
All-vendor scan:\n\
  --vendor all --mode scan   run Damiao scan + RobStride scan in one command\n\
\n\
Examples:\n\
  cargo run -p motor_cli --release -- \\\n\
    --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping\n\
\n\
  cargo run -p motor_cli --release -- \\\n\
    --vendor robstride --channel can0 --model rs-00 --motor-id 127 \\\n\
    --mode mit --pos 0.0 --vel 0.0 --kp 8 --kd 0.2 --tau 0 --loop 200 --dt-ms 20\n"
    );
}

fn verify_declared_damiao_model(
    motor: &DamiaoMotor,
    declared_model: &str,
    timeout: Duration,
    tol: f32,
) -> Result<(), String> {
    let expected = damiao_model_limits(declared_model)
        .ok_or_else(|| format!("unknown model in catalog: {declared_model}"))?;

    let pmax = motor
        .get_register_f32(21, timeout)
        .map_err(|e| format!("model handshake failed reading PMAX(rid=21): {e}"))?;
    let vmax = motor
        .get_register_f32(22, timeout)
        .map_err(|e| format!("model handshake failed reading VMAX(rid=22): {e}"))?;
    let tmax = motor
        .get_register_f32(23, timeout)
        .map_err(|e| format!("model handshake failed reading TMAX(rid=23): {e}"))?;

    let matched = match_models_by_limits(pmax, vmax, tmax, tol);
    if matched.iter().any(|m| *m == declared_model) {
        println!(
            "[ok] model handshake passed: --model {} matches PMAX/VMAX/TMAX=({:.3}, {:.3}, {:.3})",
            declared_model, pmax, vmax, tmax
        );
        return Ok(());
    }

    let suggested = suggest_models_by_limits(pmax, vmax, tmax, 3);
    let suggest_text = if suggested.is_empty() {
        "none".to_string()
    } else {
        suggested.join(", ")
    };
    Err(format!(
        "model handshake mismatch: --model {} expects ({:.3}, {:.3}, {:.3}), \
device reports ({:.3}, {:.3}, {:.3}), suggested: {}. \
If intentional, run with --verify-model 0",
        declared_model, expected.0, expected.1, expected.2, pmax, vmax, tmax, suggest_text
    ))
}

fn run_damiao(
    args: &HashMap<String, String>,
    channel: &str,
    model: &str,
    motor_id: u16,
    feedback_id: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let mode = get_str(args, "mode", "mit");
    let loop_n = get_u64(args, "loop", 1)?;
    let dt_ms = get_u64(args, "dt-ms", 20)?;
    let ensure_mode = get_u64(args, "ensure-mode", 1)? != 0;
    let set_motor_id = get_opt_u16_hex_or_dec(args, "set-motor-id")?;
    let set_feedback_id = get_opt_u16_hex_or_dec(args, "set-feedback-id")?;
    let store_after_set = get_u64(args, "store", 1)? != 0;
    let verify_id = get_u64(args, "verify-id", 1)? != 0;
    let verify_model = get_u64(args, "verify-model", 1)? != 0;
    let verify_timeout_ms = get_u64(args, "verify-timeout-ms", 500)?;
    let verify_tol = get_f32(args, "verify-tol", 0.2)?;

    let controller = DamiaoController::new_socketcan(channel)?;
    if mode == "scan" {
        let start_id = get_u16_hex_or_dec(args, "start-id", 1)?;
        let end_id = get_u16_hex_or_dec(args, "end-id", 255)?;
        if start_id == 0 || end_id == 0 || start_id > 255 || end_id > 255 || start_id > end_id {
            return Err("invalid scan range: expected 1..255 and start<=end".into());
        }
        println!(
            "[scan] probing Damiao IDs {}..{} on {}",
            start_id, end_id, channel
        );
        let mut hits = 0usize;
        for id in start_id..=end_id {
            let candidate = controller.add_motor(id, feedback_id, model)?;
            let pmax = candidate.get_register_f32(21, Duration::from_millis(80));
            let vmax = candidate.get_register_f32(22, Duration::from_millis(80));
            let tmax = candidate.get_register_f32(23, Duration::from_millis(80));
            if let (Ok(p), Ok(v), Ok(t)) = (pmax, vmax, tmax) {
                let matched = match_models_by_limits(p, v, t, 0.2);
                let model_guess = if matched.is_empty() {
                    "unknown".to_string()
                } else {
                    matched.join(",")
                };
                println!(
                    "[hit] vendor=damiao id={} model_guess={} limits=({:.3},{:.3},{:.3})",
                    id, model_guess, p, v, t
                );
                hits += 1;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        println!("[scan] done vendor=damiao hits={hits}");
        controller.close_bus()?;
        return Ok(());
    }

    let motor = controller.add_motor(motor_id, feedback_id, model)?;

    if set_motor_id.is_some() || set_feedback_id.is_some() {
        let new_motor_id = set_motor_id.unwrap_or(motor_id);
        let new_feedback_id = set_feedback_id.unwrap_or(feedback_id);
        println!(
            "[id-set] old motor_id=0x{:X} feedback_id=0x{:X} -> new motor_id=0x{:X} feedback_id=0x{:X}",
            motor_id, feedback_id, new_motor_id, new_feedback_id
        );

        if let Some(v) = set_feedback_id {
            motor.write_register_u32(7, v as u32)?;
            println!("[id-set] write rid=7 (MST_ID) = 0x{:X}", v);
        }
        if let Some(v) = set_motor_id {
            motor.write_register_u32(8, v as u32)?;
            println!("[id-set] write rid=8 (ESC_ID) = 0x{:X}", v);
        }
        if store_after_set {
            motor.store_parameters()?;
            println!("[id-set] store parameters sent");
        }
        controller.close_bus()?;

        if verify_id {
            std::thread::sleep(Duration::from_millis(120));
            let verify_ctrl = DamiaoController::new_socketcan(channel)?;
            let verify_motor = verify_ctrl.add_motor(new_motor_id, new_feedback_id, model)?;
            let esc = verify_motor.get_register_u32(8, Duration::from_millis(1000))?;
            let mst = verify_motor.get_register_u32(7, Duration::from_millis(1000))?;
            println!("[id-set] verify rid=8 (ESC_ID)=0x{:X}", esc);
            println!("[id-set] verify rid=7 (MST_ID)=0x{:X}", mst);
            verify_ctrl.close_bus()?;
            if esc != new_motor_id as u32 || mst != new_feedback_id as u32 {
                return Err(format!(
                    "id verify failed: expected ESC_ID=0x{:X}, MST_ID=0x{:X}, got ESC_ID=0x{:X}, MST_ID=0x{:X}",
                    new_motor_id, new_feedback_id, esc, mst
                )
                .into());
            }
            println!("[id-set] verify ok");
        }
        return Ok(());
    }

    if verify_model {
        verify_declared_damiao_model(
            &motor,
            model,
            Duration::from_millis(verify_timeout_ms),
            verify_tol,
        )
        .map_err(|e| e.to_string())?;
    }
    if mode != "enable" && mode != "disable" {
        controller.enable_all()?;
        std::thread::sleep(Duration::from_millis(200));
    }

    if ensure_mode && mode != "enable" && mode != "disable" {
        let cm = match mode.as_str() {
            "mit" => DamiaoControlMode::Mit,
            "pos-vel" => DamiaoControlMode::PosVel,
            "vel" => DamiaoControlMode::Vel,
            "force-pos" => DamiaoControlMode::ForcePos,
            _ => return Err(format!("unknown Damiao mode: {mode}").into()),
        };
        if let Err(e) = motor.ensure_control_mode(cm, Duration::from_millis(1000)) {
            eprintln!("[warn] ensure_mode failed: {e}");
        }
    }

    for i in 0..loop_n {
        match mode.as_str() {
            "enable" => {
                motor.enable()?;
                let _ = motor.request_motor_feedback();
            }
            "disable" => {
                motor.disable()?;
                let _ = motor.request_motor_feedback();
            }
            "mit" => {
                motor.send_cmd_mit(
                    get_f32(args, "pos", 0.0)?,
                    get_f32(args, "vel", 0.0)?,
                    get_f32(args, "kp", 30.0)?,
                    get_f32(args, "kd", 1.0)?,
                    get_f32(args, "tau", 0.0)?,
                )?;
            }
            "pos-vel" => {
                motor.send_cmd_pos_vel(get_f32(args, "pos", 0.0)?, get_f32(args, "vlim", 1.0)?)?;
            }
            "vel" => {
                motor.send_cmd_vel(get_f32(args, "vel", 0.0)?)?;
            }
            "force-pos" => {
                motor.send_cmd_force_pos(
                    get_f32(args, "pos", 0.0)?,
                    get_f32(args, "vlim", 1.0)?,
                    get_f32(args, "ratio", 0.1)?,
                )?;
            }
            _ => return Err(format!("unknown Damiao mode: {mode}").into()),
        }

        if let Some(s) = motor.latest_state() {
            println!(
                "#{i} pos={:+.3} vel={:+.3} torq={:+.3} status={}",
                s.pos, s.vel, s.torq, s.status_code
            );
        }
        std::thread::sleep(Duration::from_millis(dt_ms));
    }

    if mode == "enable" || mode == "disable" {
        controller.close_bus()?;
    } else {
        controller.shutdown()?;
    }
    Ok(())
}

fn parse_robstride_param_value(param_id: u16, raw: &str) -> Result<ParameterValue, String> {
    let info = motor_vendor_robstride::parameter_info(param_id)
        .ok_or_else(|| format!("unknown RobStride parameter 0x{param_id:04X}"))?;
    match info.data_type {
        ParameterDataType::Int8 => raw
            .parse::<i8>()
            .map(ParameterValue::I8)
            .map_err(|e| format!("invalid --param-value: {e}")),
        ParameterDataType::UInt8 => raw
            .parse::<u8>()
            .map(ParameterValue::U8)
            .map_err(|e| format!("invalid --param-value: {e}")),
        ParameterDataType::UInt16 => {
            parse_u16_hex_or_dec(raw, "param-value").map(ParameterValue::U16)
        }
        ParameterDataType::UInt32 => {
            if let Some(hex) = raw.strip_prefix("0x") {
                u32::from_str_radix(hex, 16)
                    .map(ParameterValue::U32)
                    .map_err(|e| format!("invalid --param-value: {e}"))
            } else {
                raw.parse::<u32>()
                    .map(ParameterValue::U32)
                    .map_err(|e| format!("invalid --param-value: {e}"))
            }
        }
        ParameterDataType::Float32 => raw
            .parse::<f32>()
            .map(ParameterValue::F32)
            .map_err(|e| format!("invalid --param-value: {e}")),
    }
}

fn print_robstride_param_value(param_id: u16, value: ParameterValue) {
    let name = motor_vendor_robstride::parameter_info(param_id)
        .map(|info| info.name)
        .unwrap_or("unknown");
    match value {
        ParameterValue::I8(v) => println!("param 0x{param_id:04X} ({name}) = {v}"),
        ParameterValue::U8(v) => println!("param 0x{param_id:04X} ({name}) = {v}"),
        ParameterValue::U16(v) => println!("param 0x{param_id:04X} ({name}) = {v}"),
        ParameterValue::U32(v) => println!("param 0x{param_id:04X} ({name}) = {v}"),
        ParameterValue::F32(v) => println!("param 0x{param_id:04X} ({name}) = {v:.6}"),
    }
}

fn run_robstride(
    args: &HashMap<String, String>,
    channel: &str,
    model: &str,
    motor_id: u16,
    feedback_id: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let mode = get_str(args, "mode", "ping");
    let loop_n = get_u64(args, "loop", 1)?;
    let dt_ms = get_u64(args, "dt-ms", 20)?;
    let ensure_mode = get_u64(args, "ensure-mode", 1)? != 0;
    let set_motor_id = get_opt_u16_hex_or_dec(args, "set-motor-id")?;
    let store_after_set = get_u64(args, "store", 1)? != 0;
    let controller = RobstrideController::new_socketcan(channel)?;

    if let Some((pmax, vmax, tmax)) = robstride_model_limits(model) {
        println!(
            "[info] RobStride model {} limits pmax={:.3} vmax={:.3} tmax={:.3}",
            model, pmax, vmax, tmax
        );
    }

    let query_ping = |m: &std::sync::Arc<motor_vendor_robstride::RobstrideMotor>| -> Option<u16> {
        if m.get_parameter(0x7005, Duration::from_millis(120)).is_ok() {
            return Some(0x7005);
        }
        if m.get_parameter(0x7019, Duration::from_millis(120)).is_ok() {
            return Some(0x7019);
        }
        None
    };

    if mode == "scan" {
        let start_id = get_u16_hex_or_dec(args, "start-id", 1)?;
        let end_id = get_u16_hex_or_dec(args, "end-id", 255)?;
        if start_id == 0 || end_id == 0 || start_id > 255 || end_id > 255 || start_id > end_id {
            return Err("invalid scan range: expected 1..255 and start<=end".into());
        }

        println!(
            "[scan] probing RobStride IDs {}..{} on {}",
            start_id, end_id, channel
        );
        let mut hits = 0usize;
        for id in start_id..=end_id {
            let probe_ctrl = RobstrideController::new_socketcan(channel)?;
            let candidate = probe_ctrl.add_motor(id, feedback_id, model)?;
            let mut hit = false;
            if let Ok(reply) = candidate.ping(Duration::from_millis(80)) {
                println!(
                    "[hit] vendor=robstride id={} responder_id={} model_hint={} payload={:02x?}",
                    reply.device_id, reply.responder_id, model, reply.payload
                );
                hit = true;
            } else if let Some(pid) = query_ping(&candidate) {
                println!(
                    "[hit] vendor=robstride id={} by=query-param(0x{:04X}) model_hint={}",
                    id, pid, model
                );
                hit = true;
            }
            if hit {
                hits += 1;
            }
            probe_ctrl.close_bus()?;
            std::thread::sleep(Duration::from_millis(2));
        }
        if hits == 0 {
            controller.close_bus()?;
            let fallback = RobstrideController::new_socketcan(channel)?;
            let manual_vel = get_f32(args, "manual-vel", 0.2)?;
            let manual_ms = get_u64(args, "manual-ms", 200)?;
            let manual_gap_ms = get_u64(args, "manual-gap-ms", 200)?;
            println!(
                "[scan] no ping replies; fallback to blind pulse probing (observe motor motion)"
            );
            println!(
                "[scan] pulse: vel={:.3} for {}ms, gap={}ms",
                manual_vel, manual_ms, manual_gap_ms
            );
            for id in start_id..=end_id {
                let candidate = fallback.add_motor(id, feedback_id, model)?;
                let _ = candidate.enable();
                let _ = candidate.set_mode(RobstrideControlMode::Velocity);
                let mut state_seen = false;
                let t_end = std::time::Instant::now() + Duration::from_millis(manual_ms);
                while std::time::Instant::now() < t_end {
                    let _ = candidate.set_velocity_target(manual_vel);
                    if candidate.latest_state().is_some() {
                        state_seen = true;
                    }
                    std::thread::sleep(Duration::from_millis(40));
                }
                for _ in 0..3 {
                    let _ = candidate.set_velocity_target(0.0);
                    if candidate.latest_state().is_some() {
                        state_seen = true;
                    }
                    std::thread::sleep(Duration::from_millis(30));
                }
                let _ = candidate.disable();
                if state_seen {
                    hits += 1;
                    if let Some(s) = candidate.latest_state() {
                        println!(
                            "[hit] vendor=robstride id={} by=status pos={:+.3} vel={:+.3} torq={:+.3}",
                            id, s.position, s.velocity, s.torque
                        );
                    } else {
                        println!("[hit] vendor=robstride id={} by=status", id);
                    }
                } else {
                    println!(
                        "[probe] vendor=robstride id={} model_hint={} (if this ID moved, note it)",
                        id, model
                    );
                }
                std::thread::sleep(Duration::from_millis(manual_gap_ms));
            }
            fallback.close_bus()?;
            println!("[scan] done vendor=robstride hits={hits}");
            return Ok(());
        }
        println!("[scan] done vendor=robstride hits={hits}");
        controller.close_bus()?;
        return Ok(());
    }
    let motor = controller.add_motor(motor_id, feedback_id, model)?;

    match mode.as_str() {
        "ping" => {
            if let Ok(reply) = motor.ping(Duration::from_millis(500)) {
                println!(
                    "[ok] ping device_id={} responder_id={} payload={:02x?}",
                    reply.device_id, reply.responder_id, reply.payload
                );
            } else if let Some(pid) = query_ping(&motor) {
                println!("[ok] ping(by query) param 0x{pid:04X} responded");
            } else {
                return Err(format!(
                    "robstride ping failed: no response to GET_DEVICE_ID or query parameters"
                )
                .into());
            }
            controller.close_bus()?;
            return Ok(());
        }
        "read-param" => {
            let param_id = get_u16_hex_or_dec(args, "param-id", 0)?;
            let value = motor.get_parameter(param_id, Duration::from_millis(500))?;
            print_robstride_param_value(param_id, value);
            controller.close_bus()?;
            return Ok(());
        }
        "write-param" => {
            let param_id = get_u16_hex_or_dec(args, "param-id", 0)?;
            let raw = args
                .get("param-value")
                .ok_or_else(|| "missing --param-value".to_string())?;
            let value = parse_robstride_param_value(param_id, raw)?;
            motor.write_parameter(param_id, value)?;
            std::thread::sleep(Duration::from_millis(50));
            let verify = motor.get_parameter(param_id, Duration::from_millis(500))?;
            print_robstride_param_value(param_id, verify);
            controller.close_bus()?;
            return Ok(());
        }
        _ => {}
    }

    if let Some(new_motor_id) = set_motor_id {
        motor.set_device_id(new_motor_id as u8)?;
        println!("[id-set] RobStride device id update requested: {} -> {}", motor_id, new_motor_id);
        if store_after_set {
            motor.save_parameters()?;
            println!("[id-set] RobStride save-parameters sent");
        }
        controller.close_bus()?;
        return Ok(());
    }

    if mode != "disable" {
        controller.enable_all()?;
        std::thread::sleep(Duration::from_millis(100));
    }

    if ensure_mode {
        match mode.as_str() {
            "mit" => motor.set_mode(RobstrideControlMode::Mit)?,
            "vel" => motor.set_mode(RobstrideControlMode::Velocity)?,
            _ => {}
        }
    }

    for i in 0..loop_n {
        match mode.as_str() {
            "enable" => motor.enable()?,
            "disable" => motor.disable()?,
            "mit" => {
                motor.send_cmd_mit(
                    get_f32(args, "pos", 0.0)?,
                    get_f32(args, "vel", 0.0)?,
                    get_f32(args, "kp", 8.0)?,
                    get_f32(args, "kd", 0.2)?,
                    get_f32(args, "tau", 0.0)?,
                )?;
            }
            "vel" => {
                motor.set_velocity_target(get_f32(args, "vel", 0.0)?)?;
            }
            _ => return Err(format!("unknown RobStride mode: {mode}").into()),
        }

        if let Some(s) = motor.latest_state() {
            println!(
                "#{i} pos={:+.3} vel={:+.3} torq={:+.3} temp={:.1} flags[u={} stall={} enc={} ot={} oc={} uv={}]",
                s.position,
                s.velocity,
                s.torque,
                s.temperature_c,
                s.uncalibrated,
                s.stall,
                s.magnetic_encoder_fault,
                s.overtemperature,
                s.overcurrent,
                s.undervoltage
            );
        }
        std::thread::sleep(Duration::from_millis(dt_ms));
    }

    if mode == "enable" || mode == "disable" {
        controller.close_bus()?;
    } else {
        controller.shutdown()?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
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
    let feedback_default = if vendor == "robstride" { 0x00FF } else { 0x0011 };
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
