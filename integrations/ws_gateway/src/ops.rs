use crate::{ServerConfig, Target, Transport, Vendor};
use motor_vendor_damiao::{
    match_models_by_limits, ControlMode as DamiaoControlMode, DamiaoController,
};
use motor_vendor_hexfellow::HexfellowController;
use motor_vendor_myactuator::MyActuatorController;
use motor_vendor_robstride::{
    ControlMode as RobstrideControlMode, ParameterValue as RobstrideParameterValue,
    RobstrideController, RobstrideMotor,
};
use motor_core::bus::{CanBus, CanFrame};
#[cfg(target_os = "windows")]
use motor_core::pcan::PcanBus;
#[cfg(target_os = "linux")]
use motor_core::socketcan::SocketCanBus;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};

const DAMIAO_SCAN_MODEL_HINTS: &[&str] = &[
    "4340P", "4340", "4310", "4310P", "3507", "6006", "8006", "8009", "10010L", "10010",
    "H3510", "G6215", "H6220", "JH11", "6248P",
];

fn build_scan_model_hints(preferred_model: &str) -> Vec<String> {
    let preferred = preferred_model.trim();
    // If caller provides an explicit model, scan with that model only.
    // Use full catalog only for auto/all/*.
    if !preferred.is_empty()
        && preferred.to_lowercase() != "auto"
        && preferred.to_lowercase() != "all"
        && preferred != "*"
    {
        return vec![preferred.to_string()];
    }
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
  cargo run -p ws_gateway --release -- --bind 0.0.0.0:9002\n\
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

pub(crate) fn parse_transport_in_msg(v: &Value, default: Transport) -> Result<Transport, String> {
    match v.get("transport").and_then(Value::as_str) {
        Some(s) => Transport::from_str(s),
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
    let transport = parse_transport_in_msg(v, base.transport)?;
    let start_id = as_u16(v, "start_id", 1);
    let end_id = as_u16(v, "end_id", 16);
    let feedback_base = as_u16(v, "feedback_base", 16);
    let timeout_ms = as_u64(v, "timeout_ms", 100);
    if end_id < start_id {
        return Err("end_id must be >= start_id".to_string());
    }

    let model_hints = build_scan_model_hints(&base.model);
    let controller = open_damiao_controller(base, transport)?;
    let mut hits = Vec::new();
    let mut fallback_hits = 0usize;
    for mid in start_id..=end_id {
        enum ScanHit {
            Registers { p: f32, v: f32, t: f32, fid: u16 },
            Feedback {
                fid: u16,
                status: u8,
                pos: f32,
                vel: f32,
                torq: f32,
            },
        }
        let mut found: Option<ScanHit> = None;
        let feedback_hints = build_scan_feedback_hints(feedback_base, mid);
        for fid in &feedback_hints {
            for mh in &model_hints {
                let Ok(candidate) = controller.add_motor(mid, *fid, mh) else {
                    continue;
                };
                let pmax = candidate.get_register_f32(21, Duration::from_millis(timeout_ms));
                let vmax = candidate.get_register_f32(22, Duration::from_millis(timeout_ms));
                let tmax = candidate.get_register_f32(23, Duration::from_millis(timeout_ms));
                if let (Ok(p), Ok(vv), Ok(t)) = (pmax, vmax, tmax) {
                    found = Some(ScanHit::Registers {
                        p,
                        v: vv,
                        t,
                        fid: *fid,
                    });
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
                    let Ok(candidate) = controller.add_motor(mid, *fid, mh) else {
                        continue;
                    };
                    for _ in 0..20 {
                        let _ = candidate.request_motor_feedback();
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
                        std::thread::sleep(Duration::from_millis(20));
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
                ScanHit::Registers { p, v: vv, t, fid } => {
                    let matched = match_models_by_limits(p, vv, t, 0.2);
                    let model_guess = if matched.is_empty() {
                        "unknown".to_string()
                    } else {
                        matched.join(",")
                    };
                    hits.push(json!({
                        "probe": mid,
                        "esc_id": mid,
                        "mst_id": fid,
                        "probe_feedback_id": fid,
                        "model_guess": model_guess,
                        "pmax": p,
                        "vmax": vv,
                        "tmax": t,
                        "detected_by": "registers"
                    }));
                }
                ScanHit::Feedback {
                    fid,
                    status,
                    pos,
                    vel,
                    torq,
                } => {
                    hits.push(json!({
                        "probe": mid,
                        "esc_id": mid,
                        "mst_id": fid,
                        "probe_feedback_id": fid,
                        "status": status,
                        "pos": pos,
                        "vel": vel,
                        "torq": torq,
                        "detected_by": "feedback"
                    }));
                    fallback_hits += 1;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let _ = controller.close_bus();

    Ok(json!({
        "vendor": "damiao",
        "transport": transport.as_str(),
        "count": hits.len(),
        "start_id": start_id,
        "end_id": end_id,
        "fallback_hits": fallback_hits,
        "hits": hits,
    }))
}

fn cmd_scan_robstride(v: &Value, base: &Target) -> Result<Value, String> {
    let transport = parse_transport_in_msg(v, base.transport)?;
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
                open_robstride_controller(base, transport)?;
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
        "transport": transport.as_str(),
        "count": hits.len(),
        "start_id": start_id,
        "end_id": end_id,
        "hits": hits,
    }))
}

fn myactuator_feedback_default(motor_id: u16) -> u16 {
    0x240u16.saturating_add(motor_id)
}

fn cmd_scan_myactuator(v: &Value, base: &Target) -> Result<Value, String> {
    let transport = parse_transport_in_msg(v, base.transport)?;
    let start_id = as_u16(v, "start_id", 1);
    let end_id_in = as_u16(v, "end_id", 32);
    if start_id == 0 || end_id_in == 0 || start_id > 32 || start_id > end_id_in {
        return Err("invalid scan range: expected start in 1..32 and start<=end".to_string());
    }
    let end_id = end_id_in.min(32);
    let timeout_ms = as_u64(v, "timeout_ms", 100);
    let ctrl = open_myactuator_controller(base, transport)?;
    let mut hits = Vec::new();
    for id in start_id..=end_id {
        let fid = myactuator_feedback_default(id);
        let m = match ctrl.add_motor(id, fid, &base.model) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let _ = m.request_version_date();
        if let Ok(version) = m.await_version_date(Duration::from_millis(timeout_ms)) {
            hits.push(json!({
                "probe": id,
                "motor_id": id,
                "feedback_id": fid,
                "version": version
            }));
        }
        std::thread::sleep(Duration::from_millis(3));
    }
    let _ = ctrl.close_bus();
    Ok(json!({
        "vendor": "myactuator",
        "transport": transport.as_str(),
        "count": hits.len(),
        "start_id": start_id,
        "end_id": end_id,
        "hits": hits,
    }))
}

fn cmd_scan_hexfellow(v: &Value, base: &Target) -> Result<Value, String> {
    let transport = parse_transport_in_msg(v, base.transport)?;
    let start_id = as_u16(v, "start_id", 1);
    let end_id = as_u16(v, "end_id", 32);
    let timeout_ms = as_u64(v, "timeout_ms", 200);
    let ctrl = open_hexfellow_controller(base, transport)?;
    let found = ctrl
        .scan_ids(start_id, end_id, Duration::from_millis(timeout_ms))
        .map_err(|e| e.to_string())?;
    let mut hits = Vec::new();
    for h in found {
        hits.push(json!({
            "node_id": h.node_id,
            "sw_ver": h.sw_ver,
            "peak_torque_raw": h.peak_torque_raw,
            "kp_kd_factor_raw": h.kp_kd_factor_raw,
            "dev_type": h.dev_type,
        }));
    }
    let _ = ctrl.close_bus();
    Ok(json!({
        "vendor": "hexfellow",
        "transport": transport.as_str(),
        "count": hits.len(),
        "start_id": start_id,
        "end_id": end_id,
        "hits": hits,
    }))
}

#[derive(Debug, Clone, Copy)]
struct HighTorqueStatus {
    motor_id: u16,
    pos_raw: i16,
    vel_raw: i16,
    tqe_raw: i16,
}

fn can_ext_id_for_motor(motor_id: u16) -> u32 {
    u32::from(0x8000u16 | motor_id)
}

fn send_hightorque_ext(bus: &dyn CanBus, motor_id: u16, payload: &[u8]) -> Result<(), String> {
    if payload.len() > 8 {
        return Err("payload too long (max 8 bytes)".to_string());
    }
    let mut data = [0u8; 8];
    data[..payload.len()].copy_from_slice(payload);
    bus.send(CanFrame {
        arbitration_id: can_ext_id_for_motor(motor_id),
        data,
        dlc: payload.len() as u8,
        is_extended: true,
        is_rx: false,
    })
    .map_err(|e| e.to_string())
}

fn decode_hightorque_read_reply(frame: CanFrame) -> Option<HighTorqueStatus> {
    if frame.dlc < 8 || frame.data[0] != 0x27 || frame.data[1] != 0x01 {
        return None;
    }
    let motor_id = if !frame.is_extended && (frame.arbitration_id & 0x00FF) == 0 {
        ((frame.arbitration_id >> 8) & 0x7F) as u16
    } else {
        (frame.arbitration_id & 0x7FF) as u16
    };
    Some(HighTorqueStatus {
        motor_id,
        pos_raw: i16::from_le_bytes([frame.data[2], frame.data[3]]),
        vel_raw: i16::from_le_bytes([frame.data[4], frame.data[5]]),
        tqe_raw: i16::from_le_bytes([frame.data[6], frame.data[7]]),
    })
}

fn wait_hightorque_status_for_motor(
    bus: &dyn CanBus,
    motor_id: u16,
    timeout: Duration,
) -> Result<Option<HighTorqueStatus>, String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let left = deadline.saturating_duration_since(Instant::now());
        if let Some(frame) = bus
            .recv(left.min(Duration::from_millis(20)))
            .map_err(|e| e.to_string())?
        {
            if let Some(status) = decode_hightorque_read_reply(frame) {
                if status.motor_id == motor_id {
                    return Ok(Some(status));
                }
            }
        }
    }
    Ok(None)
}

fn cmd_scan_hightorque(v: &Value, base: &Target) -> Result<Value, String> {
    let transport = parse_transport_in_msg(v, base.transport)?;
    let start_id = as_u16(v, "start_id", 1).clamp(1, 127);
    let end_id = as_u16(v, "end_id", 32).clamp(1, 127);
    if start_id > end_id {
        return Err("invalid scan range after clamp (start_id > end_id)".to_string());
    }
    let timeout_ms = as_u64(v, "timeout_ms", 80);
    let bus = open_hightorque_bus(base, transport)?;
    let mut hits = Vec::new();
    for id in start_id..=end_id {
        send_hightorque_ext(bus.as_ref(), id, &[0x17, 0x01, 0, 0, 0, 0, 0, 0])?;
        if let Some(s) =
            wait_hightorque_status_for_motor(bus.as_ref(), id, Duration::from_millis(timeout_ms))?
        {
            hits.push(json!({
                "motor_id": s.motor_id,
                "pos_raw": s.pos_raw,
                "vel_raw": s.vel_raw,
                "tqe_raw": s.tqe_raw
            }));
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let _ = bus.shutdown();
    Ok(json!({
        "vendor": "hightorque",
        "transport": transport.as_str(),
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
        Vendor::Hexfellow => cmd_scan_hexfellow(v, base),
        Vendor::Myactuator => cmd_scan_myactuator(v, base),
        Vendor::Hightorque => cmd_scan_hightorque(v, base),
    }
}

pub(crate) fn cmd_verify(v: &Value, base: &Target) -> Result<Value, String> {
    let vendor = parse_vendor_in_msg(v, base.vendor)?;
    let transport = parse_transport_in_msg(v, base.transport)?;
    let mid = as_u16(v, "motor_id", base.motor_id);
    let fid = as_u16(v, "feedback_id", base.feedback_id);
    let timeout_ms = as_u64(v, "timeout_ms", 1000);

    match vendor {
        Vendor::Damiao => {
            let preferred_model = v
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or(&base.model);
            let model_hints = build_scan_model_hints(preferred_model);
            let mut last_err: Option<String> = None;
            let mut out: Option<Value> = None;
            for model in model_hints {
                let ctrl = match open_damiao_controller(base, transport) {
                    Ok(c) => c,
                    Err(e) => {
                        last_err = Some(e.to_string());
                        continue;
                    }
                };
                let motor = match ctrl.add_motor(mid, fid, &model) {
                    Ok(m) => m,
                    Err(e) => {
                        last_err = Some(e.to_string());
                        let _ = ctrl.close_bus();
                        continue;
                    }
                };
                let esc = match motor.get_register_u32(8, Duration::from_millis(timeout_ms)) {
                    Ok(v) => v,
                    Err(e) => {
                        last_err = Some(e.to_string());
                        let _ = ctrl.close_bus();
                        continue;
                    }
                };
                let mst = match motor.get_register_u32(7, Duration::from_millis(timeout_ms)) {
                    Ok(v) => v,
                    Err(e) => {
                        last_err = Some(e.to_string());
                        let _ = ctrl.close_bus();
                        continue;
                    }
                };
                let _ = ctrl.close_bus();
                out = Some(json!({
                    "vendor": "damiao",
                    "transport": transport.as_str(),
                    "model_used": model,
                    "motor_id": mid,
                    "feedback_id": fid,
                    "esc_id": esc,
                    "mst_id": mst,
                    "ok": esc == mid as u32 && mst == fid as u32,
                }));
                break;
            }
            match out {
                Some(v) => Ok(v),
                None => Err(last_err.unwrap_or_else(|| "damiao verify failed".to_string())),
            }
        }
        Vendor::Robstride => {
            let ctrl = open_robstride_controller(base, transport)?;
            let motor = ctrl
                .add_motor(mid, fid, &base.model)
                .map_err(|e| e.to_string())?;
            let ping = motor
                .ping(Duration::from_millis(timeout_ms))
                .map_err(|e| e.to_string())?;
            let _ = ctrl.close_bus();
            Ok(json!({
                "vendor": "robstride",
                "transport": transport.as_str(),
                "motor_id": mid,
                "feedback_id": fid,
                "device_id": ping.device_id,
                "responder_id": ping.responder_id,
                "ok": ping.device_id == mid as u8,
            }))
        }
        Vendor::Hexfellow => {
            let ctrl = open_hexfellow_controller(base, transport)?;
            let motor = ctrl
                .add_motor(mid, as_u16(v, "feedback_id", base.feedback_id), &base.model)
                .map_err(|e| e.to_string())?;
            let status = motor
                .query_status(Duration::from_millis(timeout_ms))
                .map_err(|e| e.to_string())?;
            let _ = ctrl.close_bus();
            Ok(json!({
                "vendor": "hexfellow",
                "transport": transport.as_str(),
                "motor_id": mid,
                "statusword": status.statusword,
                "mode_display": status.mode_display,
                "ok": true,
            }))
        }
        Vendor::Myactuator => {
            let ctrl = open_myactuator_controller(base, transport)?;
            let fid = as_u16(v, "feedback_id", base.feedback_id);
            let eff_fid = if fid == 0 {
                myactuator_feedback_default(mid)
            } else {
                fid
            };
            let motor = ctrl.add_motor(mid, eff_fid, &base.model).map_err(|e| e.to_string())?;
            motor.request_version_date().map_err(|e| e.to_string())?;
            let version = motor
                .await_version_date(Duration::from_millis(timeout_ms))
                .map_err(|e| e.to_string())?;
            let _ = ctrl.close_bus();
            Ok(json!({
                "vendor": "myactuator",
                "transport": transport.as_str(),
                "motor_id": mid,
                "feedback_id": eff_fid,
                "version": version,
                "ok": true,
            }))
        }
        Vendor::Hightorque => {
            let bus = open_hightorque_bus(base, transport)?;
            send_hightorque_ext(bus.as_ref(), mid, &[0x17, 0x01, 0, 0, 0, 0, 0, 0])?;
            let status =
                wait_hightorque_status_for_motor(bus.as_ref(), mid, Duration::from_millis(timeout_ms))?;
            let _ = bus.shutdown();
            Ok(json!({
                "vendor": "hightorque",
                "transport": transport.as_str(),
                "motor_id": mid,
                "ok": status.is_some(),
                "state": status.map(|s| json!({"pos_raw": s.pos_raw, "vel_raw": s.vel_raw, "tqe_raw": s.tqe_raw})),
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
            let preferred_model = v
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or(&base.model);
            let model_hints = build_scan_model_hints(preferred_model);
            let mut model_used: Option<String> = None;
            let mut last_err: Option<String> = None;
            let mut motor_opt = None;
            for model in model_hints {
                match ctrl.add_motor(old_mid, old_fid, &model) {
                    Ok(m) => {
                        model_used = Some(model);
                        motor_opt = Some(m);
                        break;
                    }
                    Err(e) => {
                        last_err = Some(e.to_string());
                    }
                }
            }
            let motor = motor_opt.ok_or_else(|| {
                last_err.unwrap_or_else(|| "failed to add damiao motor for set_id".to_string())
            })?;

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
                "model_used": model_used,
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
        Vendor::Hexfellow | Vendor::Myactuator | Vendor::Hightorque => {
            Err(format!("set_id is not supported for {}", vendor.as_str()))
        }
    }
}

fn open_damiao_controller(base: &Target, transport: Transport) -> Result<DamiaoController, String> {
    match transport {
        Transport::Auto | Transport::SocketCan => {
            DamiaoController::new_socketcan(&base.channel).map_err(|e| e.to_string())
        }
        Transport::SocketCanFd => {
            DamiaoController::new_socketcanfd(&base.channel).map_err(|e| e.to_string())
        }
        Transport::DmSerial => {
            DamiaoController::new_dm_serial(&base.serial_port, base.serial_baud).map_err(|e| e.to_string())
        }
    }
}

fn open_robstride_controller(
    base: &Target,
    transport: Transport,
) -> Result<RobstrideController, String> {
    match transport {
        Transport::Auto | Transport::SocketCan => {
            RobstrideController::new_socketcan(&base.channel).map_err(|e| e.to_string())
        }
        Transport::SocketCanFd => {
            RobstrideController::new_socketcanfd(&base.channel).map_err(|e| e.to_string())
        }
        Transport::DmSerial => Err("transport dm-serial is damiao-only".to_string()),
    }
}

fn open_myactuator_controller(
    base: &Target,
    transport: Transport,
) -> Result<MyActuatorController, String> {
    match transport {
        Transport::Auto | Transport::SocketCan => {
            MyActuatorController::new_socketcan(&base.channel).map_err(|e| e.to_string())
        }
        Transport::SocketCanFd => {
            MyActuatorController::new_socketcanfd(&base.channel).map_err(|e| e.to_string())
        }
        Transport::DmSerial => Err("transport dm-serial is damiao-only".to_string()),
    }
}

fn open_hexfellow_controller(
    base: &Target,
    transport: Transport,
) -> Result<HexfellowController, String> {
    match transport {
        Transport::Auto | Transport::SocketCanFd => {
            HexfellowController::new_socketcanfd(&base.channel).map_err(|e| e.to_string())
        }
        Transport::SocketCan => Err("hexfellow requires transport socketcanfd (or auto)".to_string()),
        Transport::DmSerial => Err("transport dm-serial is damiao-only".to_string()),
    }
}

fn open_hightorque_bus(base: &Target, transport: Transport) -> Result<Box<dyn CanBus>, String> {
    match transport {
        Transport::Auto | Transport::SocketCan => {
            #[cfg(target_os = "linux")]
            {
                return Ok(Box::new(SocketCanBus::open(&base.channel).map_err(|e| e.to_string())?));
            }
            #[cfg(target_os = "windows")]
            {
                return Ok(Box::new(PcanBus::open(&base.channel).map_err(|e| e.to_string())?));
            }
            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            {
                Err("no CAN backend for current platform".to_string())
            }
        }
        Transport::SocketCanFd => Err("hightorque currently uses standard CAN transport only".to_string()),
        Transport::DmSerial => Err("transport dm-serial is damiao-only".to_string()),
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
