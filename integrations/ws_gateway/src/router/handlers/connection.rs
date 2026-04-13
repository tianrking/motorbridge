use crate::model::{ControllerHandle, MotorHandle, Vendor};
use crate::ops::{as_bool, as_u16, as_u64, parse_transport_in_msg, parse_vendor_in_msg};
use crate::session::SessionCtx;
use serde_json::{json, Value};

pub(crate) fn handle(
    op: &str,
    v: &Value,
    ctx: &mut SessionCtx,
    state_stream_enabled: &mut bool,
) -> Option<Result<Value, String>> {
    match op {
        "ping" => Some(handle_ping(v, ctx)),
        "set_target" => Some(handle_set_target(v, ctx)),
        "enable" => Some(handle_enable(ctx)),
        "disable" => Some(handle_disable(ctx)),
        "stop" => Some(handle_stop(ctx)),
        "state_once" => Some(handle_state_once(ctx)),
        "state_stream" => Some(handle_state_stream(v, state_stream_enabled)),
        "status" => Some(handle_status(ctx)),
        "poll_feedback_once" => Some(handle_poll_feedback_once(ctx)),
        "shutdown" => Some(handle_shutdown(ctx)),
        "close_bus" => Some(handle_close_bus(ctx)),
        _ => None,
    }
}

fn handle_ping(v: &Value, ctx: &mut SessionCtx) -> Result<Value, String> {
    match ctx.target.vendor {
        Vendor::Robstride => {
            ctx.ensure_connected()?;
            if let Some(MotorHandle::Robstride(m)) = ctx.motor.as_ref() {
                let p = m
                    .ping(std::time::Duration::from_millis(as_u64(v, "timeout_ms", 200)))
                    .map_err(|e| e.to_string())?;
                Ok(json!({"pong":true,"vendor":"robstride","device_id":p.device_id,"responder_id":p.responder_id}))
            } else {
                Err("motor not connected".to_string())
            }
        }
        Vendor::Damiao => Ok(json!({"pong": true, "vendor":"damiao"})),
        Vendor::Hexfellow => Ok(json!({"pong": true, "vendor":"hexfellow"})),
        Vendor::Myactuator => Ok(json!({"pong": true, "vendor":"myactuator"})),
        Vendor::Hightorque => Ok(json!({"pong": true, "vendor":"hightorque"})),
    }
}

fn handle_set_target(v: &Value, ctx: &mut SessionCtx) -> Result<Value, String> {
    let mut next = ctx.target.clone();
    next.vendor = parse_vendor_in_msg(v, next.vendor)?;
    next.transport = parse_transport_in_msg(v, next.transport)?;
    next.channel = v
        .get("channel")
        .and_then(Value::as_str)
        .unwrap_or(&next.channel)
        .to_string();
    next.serial_port = v
        .get("serial_port")
        .and_then(Value::as_str)
        .unwrap_or(&next.serial_port)
        .to_string();
    next.serial_baud = as_u64(v, "serial_baud", next.serial_baud as u64) as u32;
    next.model = v
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(&next.model)
        .to_string();
    next.motor_id = as_u16(v, "motor_id", next.motor_id);
    next.feedback_id = as_u16(v, "feedback_id", next.feedback_id);

    if next.vendor == Vendor::Robstride {
        if next.model == "4340" || next.model == "4340P" {
            next.model = "rs-00".to_string();
        }
        if next.feedback_id == 0x11 {
            next.feedback_id = 0xFF;
        }
    } else if next.vendor == Vendor::Myactuator {
        if next.model == "4340" || next.model == "4340P" {
            next.model = "X8".to_string();
        }
        if next.feedback_id == 0x11 {
            next.feedback_id = 0x241;
        }
    } else if next.vendor == Vendor::Hexfellow {
        if next.model == "4340" || next.model == "4340P" {
            next.model = "hexfellow".to_string();
        }
        if next.feedback_id == 0x11 {
            next.feedback_id = 0;
        }
    } else if next.vendor == Vendor::Hightorque {
        if next.model == "4340" || next.model == "4340P" {
            next.model = "hightorque".to_string();
        }
        if next.feedback_id == 0x11 {
            next.feedback_id = 0x01;
        }
    }

    ctx.disconnect(false);
    ctx.target = next;
    ctx.active = None;
    Ok(json!({
        "vendor": ctx.target.vendor.as_str(),
        "transport": ctx.target.transport.as_str(),
        "channel": ctx.target.channel,
        "serial_port": ctx.target.serial_port,
        "serial_baud": ctx.target.serial_baud,
        "model": ctx.target.model,
        "motor_id": ctx.target.motor_id,
        "feedback_id": ctx.target.feedback_id,
    }))
}

fn handle_enable(ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.ensure_connected()?;
    if let Some(c) = ctx.controller.as_ref() {
        match c {
            ControllerHandle::Damiao(ctrl) => ctrl.enable_all().map_err(|e| e.to_string())?,
            ControllerHandle::Hexfellow(ctrl) => ctrl.enable_all().map_err(|e| e.to_string())?,
            ControllerHandle::Hightorque(_) => {}
            ControllerHandle::Myactuator(ctrl) => ctrl.enable_all().map_err(|e| e.to_string())?,
            ControllerHandle::Robstride(ctrl) => ctrl.enable_all().map_err(|e| e.to_string())?,
        }
    }
    ctx.active = None;
    Ok(json!({"enabled": true}))
}

fn handle_disable(ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.ensure_connected()?;
    if let Some(c) = ctx.controller.as_ref() {
        match c {
            ControllerHandle::Damiao(ctrl) => ctrl.disable_all().map_err(|e| e.to_string())?,
            ControllerHandle::Hexfellow(ctrl) => ctrl.disable_all().map_err(|e| e.to_string())?,
            ControllerHandle::Hightorque(_) => {}
            ControllerHandle::Myactuator(ctrl) => ctrl.disable_all().map_err(|e| e.to_string())?,
            ControllerHandle::Robstride(ctrl) => ctrl.disable_all().map_err(|e| e.to_string())?,
        }
    }
    ctx.active = None;
    Ok(json!({"disabled": true}))
}

fn handle_stop(ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.active = None;
    if let Some(m) = ctx.motor.as_ref() {
        match m {
            MotorHandle::Damiao(mm) => mm.send_cmd_vel(0.0).map_err(|e| e.to_string())?,
            MotorHandle::Hexfellow(mm) => mm
                .command_mit(
                    motor_vendor_hexfellow::MitTarget {
                        position_rev: 0.0,
                        velocity_rev_s: 0.0,
                        torque_nm: 0.0,
                        kp: 0,
                        kd: 0,
                        limit_permille: 1000,
                    },
                    std::time::Duration::from_millis(200),
                )
                .map_err(|e| e.to_string())?,
            MotorHandle::Hightorque(mid) => {
                if let Some(ControllerHandle::Hightorque(bus)) = ctx.controller.as_ref() {
                    crate::hightorque::send_hightorque_ext(bus.as_ref(), *mid, &[0x01, 0x00, 0x00])?;
                }
            }
            MotorHandle::Myactuator(mm) => mm.stop_motor().map_err(|e| e.to_string())?,
            MotorHandle::Robstride(mm) => mm.set_velocity_target(0.0).map_err(|e| e.to_string())?,
        }
    }
    Ok(json!({"stopped": true}))
}

fn handle_state_once(ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.ensure_connected()?;
    Ok(json!({"state": ctx.build_state_snapshot()?}))
}

fn handle_state_stream(v: &Value, state_stream_enabled: &mut bool) -> Result<Value, String> {
    *state_stream_enabled = as_bool(v, "enabled", false);
    Ok(json!({"enabled": *state_stream_enabled}))
}

fn handle_status(ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.ensure_connected()?;
    match (&ctx.controller, &ctx.motor) {
        (Some(ControllerHandle::Myactuator(c)), Some(MotorHandle::Myactuator(m))) => {
            m.request_status().map_err(|e| e.to_string())?;
            m.request_multi_turn_angle().map_err(|e| e.to_string())?;
            c.poll_feedback_once().map_err(|e| e.to_string())?;
        }
        (Some(ControllerHandle::Hexfellow(_)), Some(MotorHandle::Hexfellow(_)))
        | (Some(ControllerHandle::Damiao(_)), Some(MotorHandle::Damiao(_)))
        | (Some(ControllerHandle::Robstride(_)), Some(MotorHandle::Robstride(_)))
        | (Some(ControllerHandle::Hightorque(_)), Some(MotorHandle::Hightorque(_))) => {}
        _ => return Err("motor not connected".to_string()),
    }
    Ok(json!({"state": ctx.build_state_snapshot()?}))
}

fn handle_poll_feedback_once(ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.ensure_connected()?;
    if let Some(c) = ctx.controller.as_ref() {
        match c {
            ControllerHandle::Damiao(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string())?,
            ControllerHandle::Hexfellow(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string())?,
            ControllerHandle::Hightorque(_) => {}
            ControllerHandle::Myactuator(ctrl) => {
                ctrl.poll_feedback_once().map_err(|e| e.to_string())?
            }
            ControllerHandle::Robstride(ctrl) => {
                ctrl.poll_feedback_once().map_err(|e| e.to_string())?
            }
        }
    }
    Ok(json!({"polled": true}))
}

fn handle_shutdown(ctx: &mut SessionCtx) -> Result<Value, String> {
    if let Some(c) = ctx.controller.as_ref() {
        match c {
            ControllerHandle::Damiao(ctrl) => ctrl.shutdown().map_err(|e| e.to_string())?,
            ControllerHandle::Hexfellow(ctrl) => ctrl.shutdown().map_err(|e| e.to_string())?,
            ControllerHandle::Hightorque(bus) => bus.shutdown().map_err(|e| e.to_string())?,
            ControllerHandle::Myactuator(ctrl) => ctrl.shutdown().map_err(|e| e.to_string())?,
            ControllerHandle::Robstride(ctrl) => ctrl.shutdown().map_err(|e| e.to_string())?,
        }
    }
    ctx.active = None;
    Ok(json!({"shutdown": true}))
}

fn handle_close_bus(ctx: &mut SessionCtx) -> Result<Value, String> {
    ctx.disconnect(false);
    Ok(json!({"closed": true}))
}
