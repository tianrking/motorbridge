use crate::hightorque::{send_hightorque_ext, wait_hightorque_status_for_motor};
use crate::model::{ControllerHandle, MotorHandle};
use crate::ops::{as_f32, as_u64};
use crate::session::SessionCtx;
use serde_json::{json, Value};
use std::time::Duration;

pub(crate) fn handle(op: &str, v: &Value, ctx: &mut SessionCtx) -> Option<Result<Value, String>> {
    match op {
        "current" => Some(handle_current(v, ctx)),
        "pos" => Some(handle_pos(v, ctx)),
        "version" => Some(handle_version(v, ctx)),
        "mode_query" | "mode-query" => Some(handle_mode_query(ctx)),
        "read" => Some(handle_read(v, ctx)),
        _ => None,
    }
}

fn handle_current(v: &Value, ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.ensure_connected()?;
    let current = as_f32(v, "current", 0.0);
    match ctx.motor.as_ref() {
        Some(MotorHandle::Myactuator(m)) => {
            m.send_current_setpoint(current).map_err(|e| e.to_string())?;
            Ok(json!({"op":"current", "current": current}))
        }
        Some(_) => Err("current is supported for myactuator only".to_string()),
        None => Err("motor not connected".to_string()),
    }
}

fn handle_pos(v: &Value, ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.ensure_connected()?;
    let pos = as_f32(v, "pos", 0.0);
    let max_speed = as_f32(v, "max_speed", 8.726646);
    match ctx.motor.as_ref() {
        Some(MotorHandle::Myactuator(m)) => {
            m.send_position_absolute_setpoint(pos.to_degrees(), max_speed.to_degrees())
                .map_err(|e| e.to_string())?;
            Ok(json!({"op":"pos", "pos": pos, "max_speed": max_speed}))
        }
        Some(_) => Err("pos is supported for myactuator only".to_string()),
        None => Err("motor not connected".to_string()),
    }
}

fn handle_version(v: &Value, ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.ensure_connected()?;
    let timeout_ms = as_u64(v, "timeout_ms", 500);
    match ctx.motor.as_ref() {
        Some(MotorHandle::Myactuator(m)) => {
            m.request_version_date().map_err(|e| e.to_string())?;
            let version = m
                .await_version_date(Duration::from_millis(timeout_ms))
                .map_err(|e| e.to_string())?;
            Ok(json!({"version": version}))
        }
        Some(_) => Err("version is supported for myactuator only".to_string()),
        None => Err("motor not connected".to_string()),
    }
}

fn handle_mode_query(ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.ensure_connected()?;
    match (&ctx.controller, &ctx.motor) {
        (Some(ControllerHandle::Myactuator(c)), Some(MotorHandle::Myactuator(m))) => {
            m.request_control_mode().map_err(|e| e.to_string())?;
            c.poll_feedback_once().map_err(|e| e.to_string())?;
            Ok(json!({"mode": m.latest_control_mode()}))
        }
        (Some(_), Some(_)) => Err("mode_query is supported for myactuator only".to_string()),
        _ => Err("motor not connected".to_string()),
    }
}

fn handle_read(v: &Value, ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.ensure_connected()?;
    match (&ctx.controller, &ctx.motor) {
        (Some(ControllerHandle::Hightorque(bus)), Some(MotorHandle::Hightorque(mid))) => {
            send_hightorque_ext(bus.as_ref(), *mid, &[0x17, 0x01, 0, 0, 0, 0, 0, 0])?;
            if let Some(s) = wait_hightorque_status_for_motor(
                bus.as_ref(),
                *mid,
                Duration::from_millis(as_u64(v, "timeout_ms", 500)),
            )? {
                Ok(json!({
                    "motor_id": s.motor_id,
                    "pos_raw": s.pos_raw,
                    "vel_raw": s.vel_raw,
                    "tqe_raw": s.tqe_raw,
                    "pos": s.pos_rad(),
                    "vel": s.vel_rad_s(),
                    "torq": s.tqe_raw as f32 / 100.0
                }))
            } else {
                Err("hightorque read timeout".to_string())
            }
        }
        (Some(_), Some(_)) => Err("read op is reserved for hightorque".to_string()),
        _ => Err("motor not connected".to_string()),
    }
}
