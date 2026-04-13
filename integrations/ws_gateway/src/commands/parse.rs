use crate::model::{Transport, Vendor};
use motor_vendor_damiao::ControlMode as DamiaoControlMode;
use motor_vendor_robstride::ControlMode as RobstrideControlMode;
use serde_json::Value;

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
