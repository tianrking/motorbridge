use crate::model::{Target, Vendor};
use serde_json::{json, Value};
use std::time::Duration;

use crate::vendors::damiao_ws::cmd_scan_damiao;
use crate::vendors::hightorque_ws::{send_hightorque_ext, wait_hightorque_status_for_motor};
use crate::vendors::transport_ws::{
    myactuator_feedback_default, open_hexfellow_controller, open_hightorque_bus,
    open_myactuator_controller, open_robstride_controller,
};
use super::{
    as_u16, as_u64, parse_hex_or_dec, parse_id_list_csv, parse_transport_in_msg,
    parse_vendor_in_msg,
};

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
            let ctrl = open_robstride_controller(base, transport)?;
            let motor = match ctrl.add_motor(mid, *fid, &base.model) {
                Ok(m) => m,
                Err(_) => {
                    let _ = ctrl.close_bus();
                    continue;
                }
            };
            if let Ok(p) = motor.ping(Duration::from_millis(timeout_ms)) {
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
