use crate::model::Target;
use motor_vendor_damiao::match_models_by_limits;
use serde_json::{json, Value};
use std::time::Duration;

use crate::commands::{
    as_u16, as_u64, build_scan_feedback_hints, build_scan_model_hints, parse_transport_in_msg,
};
use crate::vendors::transport_ws::open_damiao_controller;

pub(crate) fn cmd_scan_damiao(v: &Value, base: &Target) -> Result<Value, String> {
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
                ScanHit::Registers { p, v, t, fid } => {
                    let matched = match_models_by_limits(p, v, t, 0.2);
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
                        "vmax": v,
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
