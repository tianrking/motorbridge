use motor_vendor_robstride::{ParameterValue as RobstrideParameterValue, RobstrideMotor};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

use super::common::{as_bool, as_u16, as_u64, parse_u32_hex_or_dec};

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
