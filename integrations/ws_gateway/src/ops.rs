use crate::{ServerConfig, Target, Vendor};
use motor_vendor_damiao::{ControlMode as DamiaoControlMode, DamiaoController};
use motor_vendor_robstride::{
    ControlMode as RobstrideControlMode, ParameterValue as RobstrideParameterValue,
    RobstrideController, RobstrideMotor,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

fn parse_hex_or_dec(s: &str) -> Result<u16, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        u16::from_str_radix(hex, 16).map_err(|e| format!("invalid integer {s}: {e}"))
    } else {
        s.parse::<u16>()
            .map_err(|e| format!("invalid integer {s}: {e}"))
    }
}

fn parse_u32_hex_or_dec(s: &str) -> Result<u32, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).map_err(|e| format!("invalid integer {s}: {e}"))
    } else {
        s.parse::<u32>()
            .map_err(|e| format!("invalid integer {s}: {e}"))
    }
}

fn parse_id_list_csv(s: &str) -> Vec<u16> {
    s.split(',')
        .filter_map(|x| {
            let t = x.trim();
            if t.is_empty() {
                None
            } else {
                parse_hex_or_dec(t).ok()
            }
        })
        .collect()
}

pub(crate) fn parse_args() -> Result<ServerConfig, String> {
    let mut bind = "0.0.0.0:9002".to_string();
    let mut vendor = Vendor::Damiao;
    let mut channel = "can0".to_string();
    let mut model = "4340P".to_string();
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
Usage:\n\
  cargo run -p ws_gateway --release -- \\\n    --bind 0.0.0.0:9002 --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 20\n\
  cargo run -p ws_gateway --release -- \\\n    --bind 0.0.0.0:9002 --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --dt-ms 20\n"
            );
            std::process::exit(0);
        }
        let next = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {k}"))?;
        match k.as_str() {
            "--bind" => bind = next.clone(),
            "--vendor" => vendor = Vendor::from_str(next)?,
            "--channel" => channel = next.clone(),
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
    }

    Ok(ServerConfig {
        bind,
        target: Target {
            vendor,
            channel,
            model,
            motor_id,
            feedback_id,
        },
        dt_ms,
    })
}

pub(crate) fn as_bool(v: &Value, key: &str, default: bool) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(default)
}

pub(crate) fn as_u64(v: &Value, key: &str, default: u64) -> u64 {
    v.get(key).and_then(Value::as_u64).unwrap_or(default)
}

pub(crate) fn as_f32(v: &Value, key: &str, default: f32) -> f32 {
    v.get(key)
        .and_then(Value::as_f64)
        .map(|x| x as f32)
        .unwrap_or(default)
}

pub(crate) fn as_u16(v: &Value, key: &str, default: u16) -> u16 {
    match v.get(key) {
        Some(Value::Number(n)) => n.as_u64().map(|x| x as u16).unwrap_or(default),
        Some(Value::String(s)) => parse_hex_or_dec(s).unwrap_or(default),
        _ => default,
    }
}

pub(crate) fn parse_vendor_in_msg(v: &Value, default: Vendor) -> Result<Vendor, String> {
    match v.get("vendor").and_then(Value::as_str) {
        Some(s) => Vendor::from_str(s),
        None => Ok(default),
    }
}

pub(crate) fn parse_damiao_mode(v: &Value) -> Result<DamiaoControlMode, String> {
    if let Some(s) = v.get("mode").and_then(Value::as_str) {
        return match s.to_lowercase().as_str() {
            "mit" => Ok(DamiaoControlMode::Mit),
            "pos_vel" | "pos-vel" | "posvel" => Ok(DamiaoControlMode::PosVel),
            "vel" => Ok(DamiaoControlMode::Vel),
            "force_pos" | "force-pos" | "forcepos" => Ok(DamiaoControlMode::ForcePos),
            _ => Err(format!("unsupported mode string: {s}")),
        };
    }
    if let Some(n) = v.get("mode").and_then(Value::as_u64) {
        return match n {
            1 => Ok(DamiaoControlMode::Mit),
            2 => Ok(DamiaoControlMode::PosVel),
            3 => Ok(DamiaoControlMode::Vel),
            4 => Ok(DamiaoControlMode::ForcePos),
            _ => Err(format!("unsupported mode value: {n}")),
        };
    }
    Err("missing mode (string or numeric)".to_string())
}

pub(crate) fn parse_robstride_mode(v: &Value) -> Result<RobstrideControlMode, String> {
    if let Some(s) = v.get("mode").and_then(Value::as_str) {
        return match s.to_lowercase().as_str() {
            "mit" => Ok(RobstrideControlMode::Mit),
            "position" | "pos" => Ok(RobstrideControlMode::Position),
            "vel" | "velocity" => Ok(RobstrideControlMode::Velocity),
            _ => Err(format!("unsupported robstride mode string: {s}")),
        };
    }
    if let Some(n) = v.get("mode").and_then(Value::as_u64) {
        return match n {
            0 => Ok(RobstrideControlMode::Mit),
            1 => Ok(RobstrideControlMode::Position),
            2 => Ok(RobstrideControlMode::Velocity),
            _ => Err(format!("unsupported robstride mode value: {n}")),
        };
    }
    Err("missing mode (string or numeric)".to_string())
}

fn cmd_scan_damiao(v: &Value, base: &Target) -> Result<Value, String> {
    let start_id = as_u16(v, "start_id", 1);
    let end_id = as_u16(v, "end_id", 16);
    let feedback_base = as_u16(v, "feedback_base", 16);
    let timeout_ms = as_u64(v, "timeout_ms", 100);
    if end_id < start_id {
        return Err("end_id must be >= start_id".to_string());
    }

    let mut hits = Vec::new();
    for mid in start_id..=end_id {
        let fid = feedback_base + (mid & 0x0F);
        let ctrl = DamiaoController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
        let motor = match ctrl.add_motor(mid, fid, &base.model) {
            Ok(m) => m,
            Err(_) => {
                let _ = ctrl.close_bus();
                continue;
            }
        };
        let esc = motor.get_register_u32(8, Duration::from_millis(timeout_ms));
        let mst = motor.get_register_u32(7, Duration::from_millis(timeout_ms));
        if let (Ok(esc_id), Ok(mst_id)) = (esc, mst) {
            hits.push(
                json!({"probe": mid, "esc_id": esc_id, "mst_id": mst_id, "probe_feedback_id": fid}),
            );
        }
        let _ = ctrl.close_bus();
    }

    Ok(json!({
        "vendor": "damiao",
        "count": hits.len(),
        "start_id": start_id,
        "end_id": end_id,
        "hits": hits,
    }))
}

fn cmd_scan_robstride(v: &Value, base: &Target) -> Result<Value, String> {
    let start_id = as_u16(v, "start_id", 1);
    let end_id = as_u16(v, "end_id", 255);
    let timeout_ms = as_u64(v, "timeout_ms", 120);
    let param_id = as_u16(v, "param_id", 0x7019);
    if end_id < start_id {
        return Err("end_id must be >= start_id".to_string());
    }

    let feedback_ids = match v.get("feedback_ids") {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|x| {
                x.as_u64()
                    .map(|n| n as u16)
                    .or_else(|| x.as_str().and_then(|s| parse_hex_or_dec(s).ok()))
            })
            .collect::<Vec<u16>>(),
        Some(Value::String(s)) => parse_id_list_csv(s),
        _ => vec![base.feedback_id, 0xFF, 0xFE, 0x00],
    };
    let feedback_ids = if feedback_ids.is_empty() {
        vec![base.feedback_id, 0xFF, 0xFE, 0x00]
    } else {
        feedback_ids
    };

    let mut hits = Vec::new();
    for mid in start_id..=end_id {
        let mut found = None;
        for fid in &feedback_ids {
            let ctrl =
                RobstrideController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
            let motor = match ctrl.add_motor(mid, *fid, &base.model) {
                Ok(m) => m,
                Err(_) => {
                    let _ = ctrl.close_bus();
                    continue;
                }
            };
            let ping = motor.ping(Duration::from_millis(timeout_ms));
            if let Ok(p) = ping {
                found = Some(json!({
                    "probe": mid,
                    "via": "ping",
                    "feedback_id": fid,
                    "device_id": p.device_id,
                    "responder_id": p.responder_id
                }));
                let _ = ctrl.close_bus();
                break;
            }
            if let Ok(val) = motor.get_parameter_f32(param_id, Duration::from_millis(timeout_ms)) {
                found = Some(json!({
                    "probe": mid,
                    "via": "read_param",
                    "feedback_id": fid,
                    "param_id": format!("0x{param_id:04X}"),
                    "value": val
                }));
                let _ = ctrl.close_bus();
                break;
            }
            let _ = ctrl.close_bus();
        }
        if let Some(hit) = found {
            hits.push(hit);
        }
    }

    Ok(json!({
        "vendor": "robstride",
        "count": hits.len(),
        "start_id": start_id,
        "end_id": end_id,
        "hits": hits,
    }))
}

pub(crate) fn cmd_scan(v: &Value, base: &Target) -> Result<Value, String> {
    match parse_vendor_in_msg(v, base.vendor)? {
        Vendor::Damiao => cmd_scan_damiao(v, base),
        Vendor::Robstride => cmd_scan_robstride(v, base),
    }
}

pub(crate) fn cmd_verify(v: &Value, base: &Target) -> Result<Value, String> {
    let vendor = parse_vendor_in_msg(v, base.vendor)?;
    let mid = as_u16(v, "motor_id", base.motor_id);
    let fid = as_u16(v, "feedback_id", base.feedback_id);
    let timeout_ms = as_u64(v, "timeout_ms", 1000);

    match vendor {
        Vendor::Damiao => {
            let ctrl = DamiaoController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
            let motor = ctrl
                .add_motor(mid, fid, &base.model)
                .map_err(|e| e.to_string())?;
            let esc = motor
                .get_register_u32(8, Duration::from_millis(timeout_ms))
                .map_err(|e| e.to_string())?;
            let mst = motor
                .get_register_u32(7, Duration::from_millis(timeout_ms))
                .map_err(|e| e.to_string())?;
            let _ = ctrl.close_bus();
            Ok(json!({
                "vendor": "damiao",
                "motor_id": mid,
                "feedback_id": fid,
                "esc_id": esc,
                "mst_id": mst,
                "ok": esc == mid as u32 && mst == fid as u32,
            }))
        }
        Vendor::Robstride => {
            let ctrl =
                RobstrideController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
            let motor = ctrl
                .add_motor(mid, fid, &base.model)
                .map_err(|e| e.to_string())?;
            let ping = motor
                .ping(Duration::from_millis(timeout_ms))
                .map_err(|e| e.to_string())?;
            let _ = ctrl.close_bus();
            Ok(json!({
                "vendor": "robstride",
                "motor_id": mid,
                "feedback_id": fid,
                "device_id": ping.device_id,
                "responder_id": ping.responder_id,
                "ok": ping.device_id == mid as u8,
            }))
        }
    }
}

pub(crate) fn cmd_set_id(v: &Value, base: &Target) -> Result<Value, String> {
    let vendor = parse_vendor_in_msg(v, base.vendor)?;
    match vendor {
        Vendor::Damiao => {
            let old_mid = as_u16(v, "old_motor_id", base.motor_id);
            let old_fid = as_u16(v, "old_feedback_id", base.feedback_id);
            let new_mid = as_u16(v, "new_motor_id", old_mid);
            let new_fid = as_u16(v, "new_feedback_id", old_fid);
            let store = as_bool(v, "store", true);
            let verify = as_bool(v, "verify", true);
            let timeout_ms = as_u64(v, "timeout_ms", 1000);

            let ctrl = DamiaoController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
            let motor = ctrl
                .add_motor(old_mid, old_fid, &base.model)
                .map_err(|e| e.to_string())?;

            if new_fid != old_fid {
                motor
                    .write_register_u32(7, new_fid as u32)
                    .map_err(|e| e.to_string())?;
            }
            if new_mid != old_mid {
                motor
                    .write_register_u32(8, new_mid as u32)
                    .map_err(|e| e.to_string())?;
            }
            if store {
                motor.store_parameters().map_err(|e| e.to_string())?;
            }
            let _ = ctrl.close_bus();

            let mut out = json!({
                "vendor": "damiao",
                "old_motor_id": old_mid,
                "old_feedback_id": old_fid,
                "new_motor_id": new_mid,
                "new_feedback_id": new_fid,
                "store": store,
            });
            if verify {
                std::thread::sleep(Duration::from_millis(120));
                let verify_result = cmd_verify(
                    &json!({
                        "vendor":"damiao",
                        "motor_id": new_mid,
                        "feedback_id": new_fid,
                        "timeout_ms": timeout_ms
                    }),
                    base,
                )?;
                out["verify"] = verify_result;
            }
            Ok(out)
        }
        Vendor::Robstride => {
            let old_mid = as_u16(v, "old_motor_id", base.motor_id);
            let fid = as_u16(v, "feedback_id", base.feedback_id);
            let new_mid = as_u16(v, "new_motor_id", old_mid);
            if new_mid == 0 || new_mid > 0xFF {
                return Err("robstride new_motor_id must be 1..255".to_string());
            }
            let verify = as_bool(v, "verify", true);
            let timeout_ms = as_u64(v, "timeout_ms", 1000);
            let ctrl =
                RobstrideController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
            let motor = ctrl
                .add_motor(old_mid, fid, &base.model)
                .map_err(|e| e.to_string())?;
            motor
                .set_device_id(new_mid as u8)
                .map_err(|e| e.to_string())?;
            let _ = ctrl.close_bus();

            let mut out = json!({
                "vendor": "robstride",
                "old_motor_id": old_mid,
                "new_motor_id": new_mid,
                "feedback_id": fid
            });
            if verify {
                std::thread::sleep(Duration::from_millis(120));
                let verify_result = cmd_verify(
                    &json!({
                        "vendor":"robstride",
                        "motor_id": new_mid,
                        "feedback_id": fid,
                        "timeout_ms": timeout_ms
                    }),
                    base,
                )?;
                out["verify"] = verify_result;
            }
            Ok(out)
        }
    }
}

fn parse_param_type(v: &Value) -> String {
    v.get("type")
        .and_then(Value::as_str)
        .unwrap_or("f32")
        .to_lowercase()
}

fn parse_param_value(v: &Value) -> Option<String> {
    v.get("value").map(|x| match x {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => {
            if *b {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
        _ => "".to_string(),
    })
}

pub(crate) fn handle_robstride_read_param(
    motor: &Arc<RobstrideMotor>,
    v: &Value,
) -> Result<Value, String> {
    let param_id = as_u16(v, "param_id", 0x7019);
    let timeout_ms = as_u64(v, "timeout_ms", 500);
    let ty = parse_param_type(v);
    let timeout = Duration::from_millis(timeout_ms);
    match ty.as_str() {
        "i8" => Ok(
            json!({"param_id": param_id, "type":"i8", "value": motor.get_parameter_i8(param_id, timeout).map_err(|e| e.to_string())? }),
        ),
        "u8" => match motor
            .get_parameter(param_id, timeout)
            .map_err(|e| e.to_string())?
        {
            RobstrideParameterValue::U8(x) => {
                Ok(json!({"param_id": param_id, "type":"u8", "value": x }))
            }
            _ => Err(format!("parameter 0x{param_id:04X} is not u8")),
        },
        "u16" => match motor
            .get_parameter(param_id, timeout)
            .map_err(|e| e.to_string())?
        {
            RobstrideParameterValue::U16(x) => {
                Ok(json!({"param_id": param_id, "type":"u16", "value": x }))
            }
            _ => Err(format!("parameter 0x{param_id:04X} is not u16")),
        },
        "u32" => match motor
            .get_parameter(param_id, timeout)
            .map_err(|e| e.to_string())?
        {
            RobstrideParameterValue::U32(x) => {
                Ok(json!({"param_id": param_id, "type":"u32", "value": x }))
            }
            _ => Err(format!("parameter 0x{param_id:04X} is not u32")),
        },
        _ => Ok(
            json!({"param_id": param_id, "type":"f32", "value": motor.get_parameter_f32(param_id, timeout).map_err(|e| e.to_string())? }),
        ),
    }
}

pub(crate) fn handle_robstride_write_param(
    motor: &Arc<RobstrideMotor>,
    v: &Value,
) -> Result<Value, String> {
    let param_id = as_u16(v, "param_id", 0x700A);
    let timeout_ms = as_u64(v, "timeout_ms", 500);
    let verify = as_bool(v, "verify", true);
    let ty = parse_param_type(v);
    let raw = parse_param_value(v).ok_or_else(|| "missing value".to_string())?;
    let pval = match ty.as_str() {
        "i8" => RobstrideParameterValue::I8(
            parse_u32_hex_or_dec(&raw).map_err(|e| format!("invalid i8 value: {e}"))? as i8,
        ),
        "u8" => RobstrideParameterValue::U8(
            parse_u32_hex_or_dec(&raw).map_err(|e| format!("invalid u8 value: {e}"))? as u8,
        ),
        "u16" => RobstrideParameterValue::U16(
            parse_u32_hex_or_dec(&raw).map_err(|e| format!("invalid u16 value: {e}"))? as u16,
        ),
        "u32" => RobstrideParameterValue::U32(
            parse_u32_hex_or_dec(&raw).map_err(|e| format!("invalid u32 value: {e}"))?,
        ),
        _ => RobstrideParameterValue::F32(
            raw.parse::<f32>()
                .map_err(|e| format!("invalid f32 value: {e}"))?,
        ),
    };
    motor
        .write_parameter(param_id, pval)
        .map_err(|e| e.to_string())?;
    let verify_data = if verify {
        Some(handle_robstride_read_param(
            motor,
            &json!({"param_id": param_id, "type": ty, "timeout_ms": timeout_ms}),
        )?)
    } else {
        None
    };
    Ok(json!({
        "param_id": param_id,
        "type": ty,
        "value": raw,
        "verify": verify_data
    }))
}
