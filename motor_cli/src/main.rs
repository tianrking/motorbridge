use motor_vendor_damiao::{
    match_models_by_limits, model_limits, suggest_models_by_limits, ControlMode, DamiaoController,
    DamiaoMotor,
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

fn get_u16_hex_or_dec(
    args: &HashMap<String, String>,
    key: &str,
    default: u16,
) -> Result<u16, String> {
    match args.get(key) {
        Some(v) => {
            let parsed = if let Some(hex) = v.strip_prefix("0x") {
                u16::from_str_radix(hex, 16).map_err(|e| format!("invalid --{key}: {e}"))?
            } else {
                v.parse::<u16>()
                    .map_err(|e| format!("invalid --{key}: {e}"))?
            };
            Ok(parsed)
        }
        None => Ok(default),
    }
}

fn print_help() {
    println!(
        "motor_cli\n\
Usage:\n\
  cargo run -p motor_cli --release -- \\\n    --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \\\n    --mode mit --pos 0 --vel 0 --kp 30 --kd 1 --tau 0 --loop 200 --dt-ms 20\n\n\
Modes:\n\
  --mode enable    (send enable command only)\n\
  --mode disable   (send disable command only)\n\
  --mode mit       (MIT: pos/vel/kp/kd/tau)\n\
  --mode pos-vel   (Position mode: pos + vlim)\n\
  --mode vel       (Velocity mode: vel)\n\
  --mode force-pos (Force-position: pos + vlim + ratio)\n\n\
Common args:\n\
  --channel      default can0\n\
  --model        default 4340\n\
  --motor-id     default 0x01\n\
  --feedback-id  default 0x11\n\
  --loop         send cycles, default 1\n\
  --dt-ms        period ms, default 20\n\
  --ensure-mode  1/0, default 1\n\n\
  --verify-model 1/0, default 1 (read PMAX/VMAX/TMAX registers and verify --model)\n\
  --verify-timeout-ms  register read timeout, default 500\n\
  --verify-tol   absolute tolerance for PMAX/VMAX/TMAX compare, default 0.2\n\n\
MIT args:\n\
  --pos --vel --kp --kd --tau\n\n\
POS_VEL args:\n\
  --pos --vlim\n\n\
VEL args:\n\
  --vel\n\n\
FORCE_POS args:\n\
  --pos --vlim --ratio\n"
    );
}

fn verify_declared_model(
    motor: &DamiaoMotor,
    declared_model: &str,
    timeout: Duration,
    tol: f32,
) -> Result<(), String> {
    let expected = model_limits(declared_model)
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();
    if args.contains_key("help") {
        print_help();
        return Ok(());
    }

    let channel = get_str(&args, "channel", "can0");
    let model = get_str(&args, "model", "4340");
    let motor_id = get_u16_hex_or_dec(&args, "motor-id", 0x01)?;
    let feedback_id = get_u16_hex_or_dec(&args, "feedback-id", 0x11)?;
    let mode = get_str(&args, "mode", "mit");
    let loop_n = get_u64(&args, "loop", 1)?;
    let dt_ms = get_u64(&args, "dt-ms", 20)?;
    let ensure_mode = get_u64(&args, "ensure-mode", 1)? != 0;
    let verify_model = get_u64(&args, "verify-model", 1)? != 0;
    let verify_timeout_ms = get_u64(&args, "verify-timeout-ms", 500)?;
    let verify_tol = get_f32(&args, "verify-tol", 0.2)?;

    println!(
        "channel={} model={} motor_id=0x{:X} feedback_id=0x{:X} mode={}",
        channel, model, motor_id, feedback_id, mode
    );
    if mode == "enable" || mode == "disable" {
        eprintln!("[info] enable/disable status feedback can be delayed; use larger --loop/--dt-ms to observe transitions.");
    }

    let controller = DamiaoController::new_socketcan(&channel)?;
    let motor = controller.add_motor(motor_id, feedback_id, &model)?;
    if verify_model {
        verify_declared_model(
            &motor,
            &model,
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
            "mit" => ControlMode::Mit,
            "pos-vel" => ControlMode::PosVel,
            "vel" => ControlMode::Vel,
            "force-pos" => ControlMode::ForcePos,
            _ => return Err(format!("unknown mode: {mode}").into()),
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
                let pos = get_f32(&args, "pos", 0.0)?;
                let vel = get_f32(&args, "vel", 0.0)?;
                let kp = get_f32(&args, "kp", 30.0)?;
                let kd = get_f32(&args, "kd", 1.0)?;
                let tau = get_f32(&args, "tau", 0.0)?;
                motor.send_cmd_mit(pos, vel, kp, kd, tau)?;
            }
            "pos-vel" => {
                let pos = get_f32(&args, "pos", 0.0)?;
                let vlim = get_f32(&args, "vlim", 1.0)?;
                motor.send_cmd_pos_vel(pos, vlim)?;
            }
            "vel" => {
                let vel = get_f32(&args, "vel", 0.0)?;
                motor.send_cmd_vel(vel)?;
            }
            "force-pos" => {
                let pos = get_f32(&args, "pos", 0.0)?;
                let vlim = get_f32(&args, "vlim", 1.0)?;
                let ratio = get_f32(&args, "ratio", 0.1)?;
                motor.send_cmd_force_pos(pos, vlim, ratio)?;
            }
            _ => return Err(format!("unknown mode: {mode}").into()),
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
        // Keep commanded state; only close local bus/session.
        controller.close_bus()?;
    } else {
        controller.shutdown()?;
    }
    Ok(())
}
