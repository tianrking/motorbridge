use crate::{MotorHandle, MotorHandleInner};
use motor_vendor_robstride::ParameterValue;
use std::time::Duration;

pub(crate) fn get_i8(motor: &mut MotorHandle, param_id: u16, timeout_ms: u32) -> Result<i8, String> {
    match get_value(motor, param_id, timeout_ms)? {
        ParameterValue::I8(v) => Ok(v),
        _ => Err("RobStride parameter type mismatch".to_string()),
    }
}

pub(crate) fn get_u8(motor: &mut MotorHandle, param_id: u16, timeout_ms: u32) -> Result<u8, String> {
    match get_value(motor, param_id, timeout_ms)? {
        ParameterValue::U8(v) => Ok(v),
        _ => Err("RobStride parameter type mismatch".to_string()),
    }
}

pub(crate) fn get_u16(motor: &mut MotorHandle, param_id: u16, timeout_ms: u32) -> Result<u16, String> {
    match get_value(motor, param_id, timeout_ms)? {
        ParameterValue::U16(v) => Ok(v),
        _ => Err("RobStride parameter type mismatch".to_string()),
    }
}

pub(crate) fn get_u32(motor: &mut MotorHandle, param_id: u16, timeout_ms: u32) -> Result<u32, String> {
    match get_value(motor, param_id, timeout_ms)? {
        ParameterValue::U32(v) => Ok(v),
        _ => Err("RobStride parameter type mismatch".to_string()),
    }
}

pub(crate) fn get_f32(motor: &mut MotorHandle, param_id: u16, timeout_ms: u32) -> Result<f32, String> {
    match get_value(motor, param_id, timeout_ms)? {
        ParameterValue::F32(v) => Ok(v),
        _ => Err("RobStride parameter type mismatch".to_string()),
    }
}

pub(crate) fn write_i8(motor: &mut MotorHandle, param_id: u16, value: i8) -> Result<(), String> {
    write_value(motor, param_id, ParameterValue::I8(value))
}

pub(crate) fn write_u8(motor: &mut MotorHandle, param_id: u16, value: u8) -> Result<(), String> {
    write_value(motor, param_id, ParameterValue::U8(value))
}

pub(crate) fn write_u16(motor: &mut MotorHandle, param_id: u16, value: u16) -> Result<(), String> {
    write_value(motor, param_id, ParameterValue::U16(value))
}

pub(crate) fn write_u32(motor: &mut MotorHandle, param_id: u16, value: u32) -> Result<(), String> {
    write_value(motor, param_id, ParameterValue::U32(value))
}

pub(crate) fn write_f32(motor: &mut MotorHandle, param_id: u16, value: f32) -> Result<(), String> {
    write_value(motor, param_id, ParameterValue::F32(value))
}

fn get_value(motor: &mut MotorHandle, param_id: u16, timeout_ms: u32) -> Result<ParameterValue, String> {
    match &motor.inner {
        MotorHandleInner::Robstride(m) => m
            .get_parameter(param_id, Duration::from_millis(timeout_ms as u64))
            .map_err(|e| e.to_string()),
        _ => Err("RobStride parameter access requires a RobStride motor".to_string()),
    }
}

fn write_value(motor: &mut MotorHandle, param_id: u16, value: ParameterValue) -> Result<(), String> {
    match &motor.inner {
        MotorHandleInner::Robstride(m) => m.write_parameter(param_id, value).map_err(|e| e.to_string()),
        _ => Err("RobStride parameter access requires a RobStride motor".to_string()),
    }
}
