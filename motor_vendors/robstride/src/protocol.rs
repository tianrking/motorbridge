use crate::registers::{parameter_info, ParameterDataType};
use motor_core::error::{MotorError, Result};

pub struct CommunicationType;

impl CommunicationType {
    pub const GET_DEVICE_ID: u32 = 0;
    pub const OPERATION_CONTROL: u32 = 1;
    pub const OPERATION_STATUS: u32 = 2;
    pub const ENABLE: u32 = 3;
    pub const DISABLE: u32 = 4;
    pub const SET_ZERO_POSITION: u32 = 6;
    pub const SET_DEVICE_ID: u32 = 7;
    pub const READ_PARAMETER: u32 = 17;
    pub const WRITE_PARAMETER: u32 = 18;
    pub const FAULT_REPORT: u32 = 21;
    pub const SAVE_PARAMETERS: u32 = 22;
    pub const SET_BAUDRATE: u32 = 23;
    pub const ACTIVE_REPORT: u32 = 24;
    pub const SET_PROTOCOL: u32 = 25;
}

#[derive(Debug, Clone, Copy)]
pub struct StatusFlags {
    pub uncalibrated: bool,
    pub stall: bool,
    pub magnetic_encoder_fault: bool,
    pub overtemperature: bool,
    pub overcurrent: bool,
    pub undervoltage: bool,
    pub device_id: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct StatusFrame {
    pub flags: StatusFlags,
    pub position: f32,
    pub velocity: f32,
    pub torque: f32,
    pub temperature_c: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct PingReply {
    pub device_id: u8,
    pub responder_id: u8,
    pub payload: [u8; 8],
}

pub fn build_ext_id(comm_type: u32, extra_data: u16, node_id: u8) -> u32 {
    (comm_type << 24) | (u32::from(extra_data) << 8) | u32::from(node_id)
}

pub fn ext_id_parts(arbitration_id: u32) -> (u32, u16, u8) {
    (
        (arbitration_id >> 24) & 0x1F,
        ((arbitration_id >> 8) & 0xFFFF) as u16,
        (arbitration_id & 0xFF) as u8,
    )
}

pub fn encode_parameter_read(param_id: u16) -> [u8; 8] {
    let mut out = [0u8; 8];
    out[0..2].copy_from_slice(&param_id.to_le_bytes());
    out
}

pub fn encode_parameter_write(param_id: u16, raw_value: [u8; 4]) -> [u8; 8] {
    let mut out = [0u8; 8];
    out[0..2].copy_from_slice(&param_id.to_le_bytes());
    out[4..8].copy_from_slice(&raw_value);
    out
}

pub fn encode_mit_command(
    position: f32,
    velocity: f32,
    kp: f32,
    kd: f32,
    torque: f32,
    pmax: f32,
    vmax: f32,
    tmax: f32,
    kp_max: f32,
    kd_max: f32,
) -> (u16, [u8; 8]) {
    let pos_u16 = (((position.clamp(-pmax, pmax) / pmax) + 1.0) * 0x7FFF as f32) as u16;
    let vel_u16 = (((velocity.clamp(-vmax, vmax) / vmax) + 1.0) * 0x7FFF as f32) as u16;
    let kp_u16 = ((kp.clamp(0.0, kp_max) / kp_max) * 0xFFFF as f32) as u16;
    let kd_u16 = ((kd.clamp(0.0, kd_max) / kd_max) * 0xFFFF as f32) as u16;
    let torque_u16 = (((torque.clamp(-tmax, tmax) / tmax) + 1.0) * 0x7FFF as f32) as u16;
    let data = [
        (pos_u16 >> 8) as u8,
        pos_u16 as u8,
        (vel_u16 >> 8) as u8,
        vel_u16 as u8,
        (kp_u16 >> 8) as u8,
        kp_u16 as u8,
        (kd_u16 >> 8) as u8,
        kd_u16 as u8,
    ];
    (torque_u16, data)
}

pub fn decode_ping_reply(arbitration_id: u32, data: [u8; 8]) -> Result<PingReply> {
    let (comm_type, extra_data, responder_id) = ext_id_parts(arbitration_id);
    if comm_type != CommunicationType::GET_DEVICE_ID {
        return Err(MotorError::Protocol("not a ping reply".to_string()));
    }
    Ok(PingReply {
        device_id: (extra_data & 0xFF) as u8,
        responder_id,
        payload: data,
    })
}

pub fn decode_read_parameter_value(param_id: u16, payload: [u8; 8]) -> Result<[u8; 4]> {
    if parameter_info(param_id).is_none() {
        return Err(MotorError::InvalidArgument(format!(
            "unknown RobStride parameter 0x{param_id:04X}"
        )));
    }
    Ok([payload[4], payload[5], payload[6], payload[7]])
}

pub fn decode_status_frame(
    extra_data: u16,
    data: [u8; 8],
    pmax: f32,
    vmax: f32,
    tmax: f32,
) -> StatusFrame {
    let position_u16 = u16::from_be_bytes([data[0], data[1]]);
    let velocity_u16 = u16::from_be_bytes([data[2], data[3]]);
    let torque_u16 = u16::from_be_bytes([data[4], data[5]]);
    let temperature_u16 = u16::from_be_bytes([data[6], data[7]]);

    let flags = StatusFlags {
        uncalibrated: ((extra_data >> 13) & 0x01) != 0,
        stall: ((extra_data >> 12) & 0x01) != 0,
        magnetic_encoder_fault: ((extra_data >> 11) & 0x01) != 0,
        overtemperature: ((extra_data >> 10) & 0x01) != 0,
        overcurrent: ((extra_data >> 9) & 0x01) != 0,
        undervoltage: ((extra_data >> 8) & 0x01) != 0,
        device_id: (extra_data & 0xFF) as u8,
    };

    StatusFrame {
        flags,
        position: (f32::from(position_u16) / 0x7FFF as f32 - 1.0) * pmax,
        velocity: (f32::from(velocity_u16) / 0x7FFF as f32 - 1.0) * vmax,
        torque: (f32::from(torque_u16) / 0x7FFF as f32 - 1.0) * tmax,
        temperature_c: f32::from(temperature_u16) * 0.1,
    }
}

pub fn encode_parameter_value(
    param_id: u16,
    value: crate::motor::ParameterValue,
) -> Result<[u8; 4]> {
    let info = parameter_info(param_id).ok_or_else(|| {
        MotorError::InvalidArgument(format!("unknown RobStride parameter 0x{param_id:04X}"))
    })?;
    let raw = match (info.data_type, value) {
        (ParameterDataType::Int8, crate::motor::ParameterValue::I8(v)) => [v as u8, 0, 0, 0],
        (ParameterDataType::UInt8, crate::motor::ParameterValue::U8(v)) => [v, 0, 0, 0],
        (ParameterDataType::UInt16, crate::motor::ParameterValue::U16(v)) => {
            let mut out = [0u8; 4];
            out[0..2].copy_from_slice(&v.to_le_bytes());
            out
        }
        (ParameterDataType::UInt32, crate::motor::ParameterValue::U32(v)) => v.to_le_bytes(),
        (ParameterDataType::Float32, crate::motor::ParameterValue::F32(v)) => v.to_le_bytes(),
        _ => {
            return Err(MotorError::InvalidArgument(format!(
                "type mismatch for parameter 0x{param_id:04X}"
            )))
        }
    };
    Ok(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motor::ParameterValue;

    #[test]
    fn ext_id_build_and_parse_roundtrip() {
        let id = build_ext_id(CommunicationType::READ_PARAMETER, 0xABCD, 0x7F);
        let (comm, extra, node) = ext_id_parts(id);
        assert_eq!(comm, CommunicationType::READ_PARAMETER);
        assert_eq!(extra, 0xABCD);
        assert_eq!(node, 0x7F);
    }

    #[test]
    fn ping_reply_decode_validates_comm_type() {
        let ok_id = build_ext_id(CommunicationType::GET_DEVICE_ID, 0x007F, 0x55);
        let bad_id = build_ext_id(CommunicationType::READ_PARAMETER, 0x007F, 0x55);
        let payload = [1, 2, 3, 4, 5, 6, 7, 8];

        let reply = decode_ping_reply(ok_id, payload).expect("valid ping reply");
        assert_eq!(reply.device_id, 0x7F);
        assert_eq!(reply.responder_id, 0x55);
        assert_eq!(reply.payload, payload);
        assert!(decode_ping_reply(bad_id, payload).is_err());
    }

    #[test]
    fn parameter_encoding_checks_type() {
        let f = encode_parameter_value(0x7019, ParameterValue::F32(1.25)).expect("f32 param");
        assert_eq!(f, 1.25f32.to_le_bytes());

        let u32v = 123_456u32;
        let u = encode_parameter_value(0x7028, ParameterValue::U32(u32v)).expect("u32 param");
        assert_eq!(u, u32v.to_le_bytes());

        let mismatch = encode_parameter_value(0x7028, ParameterValue::F32(1.0));
        assert!(mismatch.is_err());
    }

    #[test]
    fn read_parameter_decode_rejects_unknown_id() {
        let payload = [0u8; 8];
        let ok = decode_read_parameter_value(0x7019, payload).expect("known param");
        assert_eq!(ok, [0, 0, 0, 0]);
        assert!(decode_read_parameter_value(0xDEAD, payload).is_err());
    }
}
