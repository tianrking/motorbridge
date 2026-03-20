use crate::args::{get_f32, get_opt_u16_hex_or_dec, get_str, get_u16_hex_or_dec, get_u64};
use motor_vendor_damiao::{
    match_models_by_limits, model_limits as damiao_model_limits, suggest_models_by_limits,
    ControlMode as DamiaoControlMode, DamiaoController, DamiaoMotor,
};
use std::collections::HashMap;
use std::time::Duration;

const DAMIAO_SCAN_MODEL_HINTS: &[&str] = &[
    "4340P", "4340", "4310", "4310P", "3507", "6006", "8006", "8009", "10010L", "10010", "H3510",
    "G6215", "H6220", "JH11", "6248P",
];

fn build_scan_model_hints() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for m in DAMIAO_SCAN_MODEL_HINTS {
        if !out.iter().any(|x| x.eq_ignore_ascii_case(m)) {
            out.push((*m).to_string());
        }
    }
    out
}

fn build_scan_feedback_hints(base_feedback_id: u16, motor_id: u16) -> Vec<u16> {
    let mut out = Vec::new();
    let inferred = motor_id.saturating_add(0x10);
    for fid in [inferred, base_feedback_id, 0x0011, 0x0017] {
        if !out.contains(&fid) {
            out.push(fid);
        }
    }
    out
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

pub fn run_damiao(
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
        let model_hints = build_scan_model_hints();
        println!(
            "[scan] probing Damiao IDs {}..{} on {}",
            start_id, end_id, channel
        );
        let mut hits = 0usize;
        let mut fallback_hits = 0usize;
        for id in start_id..=end_id {
            let feedback_hints = build_scan_feedback_hints(feedback_id, id);
            enum ScanHit {
                Registers {
                    p: f32,
                    v: f32,
                    t: f32,
                    fid: u16,
                },
                Feedback {
                    fid: u16,
                    status: u8,
                    pos: f32,
                    vel: f32,
                    torq: f32,
                },
            }
            let mut found: Option<ScanHit> = None;
            for fid in &feedback_hints {
                for mh in &model_hints {
                    let Ok(candidate) = controller.add_motor(id, *fid, mh) else {
                        continue;
                    };
                    let pmax = candidate.get_register_f32(21, Duration::from_millis(120));
                    let vmax = candidate.get_register_f32(22, Duration::from_millis(120));
                    let tmax = candidate.get_register_f32(23, Duration::from_millis(120));
                    if let (Ok(p), Ok(v), Ok(t)) = (pmax, vmax, tmax) {
                        found = Some(ScanHit::Registers { p, v, t, fid: *fid });
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            if found.is_none() {
                for fid in &feedback_hints {
                    for mh in &model_hints {
                        let Ok(candidate) = controller.add_motor(id, *fid, mh) else {
                            continue;
                        };
                        let _ = candidate.request_motor_feedback();
                        for _ in 0..4 {
                            let _ = controller.poll_feedback_once();
                            if let Some(s) = candidate.latest_state() {
                                found = Some(ScanHit::Feedback {
                                    fid: *fid,
                                    status: s.status_code,
                                    pos: s.pos,
                                    vel: s.vel,
                                    torq: s.torq,
                                });
                                break;
                            }
                            std::thread::sleep(Duration::from_millis(8));
                        }
                        if found.is_some() {
                            break;
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }
            }
            if let Some(hit) = found {
                match hit {
                    ScanHit::Registers { p, v, t, fid } => {
                        let matched = match_models_by_limits(p, v, t, 0.2);
                        let model_guess = if matched.is_empty() {
                            "unknown".to_string()
                        } else {
                            matched.join(",")
                        };
                        println!(
                            "[hit] vendor=damiao id={} feedback_id=0x{:X} model_guess={} limits=({:.3},{:.3},{:.3})",
                            id, fid, model_guess, p, v, t
                        );
                        hits += 1;
                    }
                    ScanHit::Feedback {
                        fid,
                        status,
                        pos,
                        vel,
                        torq,
                    } => {
                        println!(
                            "[hit] vendor=damiao id={} feedback_id=0x{:X} detected_by=feedback status={} pos={:+.3} vel={:+.3} torq={:+.3}",
                            id, fid, status, pos, vel, torq
                        );
                        hits += 1;
                        fallback_hits += 1;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        if fallback_hits > 0 {
            println!(
                "[scan] fallback feedback-detection hits={fallback_hits} (register read unavailable on some motors)"
            );
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
        controller.close_bus()?;

        // Reconnect using NEW IDs before store/verify.
        // Otherwise a store sent via an old-ID handle may be lost.
        if store_after_set || verify_id {
            std::thread::sleep(Duration::from_millis(120));
            let verify_ctrl = DamiaoController::new_socketcan(channel)?;
            let verify_motor = verify_ctrl.add_motor(new_motor_id, new_feedback_id, model)?;

            if store_after_set {
                verify_motor.store_parameters()?;
                println!("[id-set] store parameters sent (via new id)");
                std::thread::sleep(Duration::from_millis(120));
            }

            if verify_id {
                let esc = verify_motor.get_register_u32(8, Duration::from_millis(1000))?;
                let mst = verify_motor.get_register_u32(7, Duration::from_millis(1000))?;
                println!("[id-set] verify rid=8 (ESC_ID)=0x{:X}", esc);
                println!("[id-set] verify rid=7 (MST_ID)=0x{:X}", mst);
                if esc != new_motor_id as u32 || mst != new_feedback_id as u32 {
                    verify_ctrl.close_bus()?;
                    return Err(format!(
                        "id verify failed: expected ESC_ID=0x{:X}, MST_ID=0x{:X}, got ESC_ID=0x{:X}, MST_ID=0x{:X}",
                        new_motor_id, new_feedback_id, esc, mst
                    )
                    .into());
                }
                println!("[id-set] verify ok");
            }
            verify_ctrl.close_bus()?;
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

#[cfg(test)]
mod tests {
    use super::{build_scan_feedback_hints, build_scan_model_hints};

    #[test]
    fn scan_model_hints_are_unique() {
        let hints = build_scan_model_hints();
        assert!(!hints.is_empty());
        let count_4310 = hints.iter().filter(|m| m.as_str() == "4310").count();
        assert_eq!(count_4310, 1);
        assert!(hints.iter().any(|m| m == "4340P"));
    }

    #[test]
    fn scan_feedback_hints_include_common_ids() {
        let fids = build_scan_feedback_hints(0x0017, 0x0007);
        assert!(fids.contains(&0x0011));
        assert!(fids.contains(&0x0017));
        assert_eq!(fids[0], 0x0017);
    }
}
