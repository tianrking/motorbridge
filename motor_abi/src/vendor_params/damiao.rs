use crate::{MotorHandle, MotorHandleInner};
use std::time::Duration;

fn damiao_param_rid(param_id: u16) -> Result<u8, String> {
    if param_id <= u8::MAX as u16 {
        Ok(param_id as u8)
    } else {
        Err("Damiao parameter/register id must be in 0..=255".to_string())
    }
}

pub(crate) fn get_param_f32(
    motor: &mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
) -> Result<f32, String> {
    let rid = damiao_param_rid(param_id)?;
    match &motor.inner {
        MotorHandleInner::Damiao(m) => m
            .get_register_f32(rid, Duration::from_millis(timeout_ms as u64))
            .map_err(|e| e.to_string()),
        _ => Err("Damiao parameter access requires a Damiao motor".to_string()),
    }
}

pub(crate) fn get_param_u32(
    motor: &mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
) -> Result<u32, String> {
    let rid = damiao_param_rid(param_id)?;
    match &motor.inner {
        MotorHandleInner::Damiao(m) => m
            .get_register_u32(rid, Duration::from_millis(timeout_ms as u64))
            .map_err(|e| e.to_string()),
        _ => Err("Damiao parameter access requires a Damiao motor".to_string()),
    }
}

pub(crate) fn write_param_f32(motor: &mut MotorHandle, param_id: u16, value: f32) -> Result<(), String> {
    let rid = damiao_param_rid(param_id)?;
    match &motor.inner {
        MotorHandleInner::Damiao(m) => m.write_register_f32(rid, value).map_err(|e| e.to_string()),
        _ => Err("Damiao parameter access requires a Damiao motor".to_string()),
    }
}

pub(crate) fn write_param_u32(motor: &mut MotorHandle, param_id: u16, value: u32) -> Result<(), String> {
    let rid = damiao_param_rid(param_id)?;
    match &motor.inner {
        MotorHandleInner::Damiao(m) => m.write_register_u32(rid, value).map_err(|e| e.to_string()),
        _ => Err("Damiao parameter access requires a Damiao motor".to_string()),
    }
}
