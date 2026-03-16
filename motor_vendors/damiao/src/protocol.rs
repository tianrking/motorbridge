use motor_core::error::{MotorError, Result};

pub const KP_MIN: f32 = 0.0;
pub const KP_MAX: f32 = 500.0;
pub const KD_MIN: f32 = 0.0;
pub const KD_MAX: f32 = 5.0;

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub p_min: f32,
    pub p_max: f32,
    pub v_min: f32,
    pub v_max: f32,
    pub t_min: f32,
    pub t_max: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct SensorFeedback {
    pub can_id: u8,
    pub status_code: u8,
    pub pos: f32,
    pub vel: f32,
    pub torq: f32,
    pub t_mos: f32,
    pub t_rotor: f32,
}

pub fn status_name(status: u8) -> &'static str {
    match status {
        0x0 => "DISABLED",
        0x1 => "ENABLED",
        0x8 => "OVER_VOLTAGE",
        0x9 => "UNDER_VOLTAGE",
        0xA => "OVER_CURRENT",
        0xB => "MOS_OVER_TEMP",
        0xC => "ROTOR_OVER_TEMP",
        0xD => "LOST_COMM",
        0xE => "OVERLOAD",
        _ => "UNKNOWN",
    }
}

pub fn float_to_uint(x: f32, x_min: f32, x_max: f32, bits: u8) -> u32 {
    let span = x_max - x_min;
    let clipped = x.clamp(x_min, x_max);
    ((clipped - x_min) * (((1u32 << bits) - 1) as f32) / span) as u32
}

pub fn uint_to_float(x: u32, x_min: f32, x_max: f32, bits: u8) -> f32 {
    let span = x_max - x_min;
    (x as f32) * span / (((1u32 << bits) - 1) as f32) + x_min
}

pub fn encode_mit_cmd(pos: f32, vel: f32, torq: f32, kp: f32, kd: f32, limits: Limits) -> [u8; 8] {
    let pos_u = float_to_uint(pos, limits.p_min, limits.p_max, 16);
    let vel_u = float_to_uint(vel, limits.v_min, limits.v_max, 12);
    let kp_u = float_to_uint(kp, KP_MIN, KP_MAX, 12);
    let kd_u = float_to_uint(kd, KD_MIN, KD_MAX, 12);
    let torq_u = float_to_uint(torq, limits.t_min, limits.t_max, 12);

    [
        ((pos_u >> 8) & 0xFF) as u8,
        (pos_u & 0xFF) as u8,
        ((vel_u >> 4) & 0xFF) as u8,
        (((vel_u & 0xF) << 4) | ((kp_u >> 8) & 0xF)) as u8,
        (kp_u & 0xFF) as u8,
        ((kd_u >> 4) & 0xFF) as u8,
        (((kd_u & 0xF) << 4) | ((torq_u >> 8) & 0xF)) as u8,
        (torq_u & 0xFF) as u8,
    ]
}

pub fn encode_pos_vel_cmd(target_position: f32, velocity_limit: f32) -> [u8; 8] {
    let mut out = [0u8; 8];
    out[0..4].copy_from_slice(&target_position.to_le_bytes());
    out[4..8].copy_from_slice(&velocity_limit.to_le_bytes());
    out
}

pub fn encode_vel_cmd(target_velocity: f32) -> [u8; 8] {
    let mut out = [0u8; 8];
    out[0..4].copy_from_slice(&target_velocity.to_le_bytes());
    out
}

pub fn encode_force_pos_cmd(
    target_position: f32,
    velocity_limit: f32,
    torque_limit_ratio: f32,
) -> [u8; 8] {
    let v_des = (velocity_limit.clamp(0.0, 100.0) * 100.0) as u16;
    let i_des = (torque_limit_ratio.clamp(0.0, 1.0) * 10_000.0) as u16;
    let mut out = [0u8; 8];
    out[0..4].copy_from_slice(&target_position.to_le_bytes());
    out[4..6].copy_from_slice(&v_des.min(10_000).to_le_bytes());
    out[6..8].copy_from_slice(&i_des.min(10_000).to_le_bytes());
    out
}

pub fn is_register_reply(data: &[u8; 8]) -> bool {
    let rid = data[3];
    data[1] <= 0x0F && data[2] == 0x33 && crate::registers::register_info(rid).is_some()
}

pub fn decode_sensor_feedback(data: [u8; 8], limits: Limits) -> SensorFeedback {
    let can_id = data[0] & 0x0F;
    let status = data[0] >> 4;
    let pos_int = ((data[1] as u32) << 8) | (data[2] as u32);
    let vel_int = ((data[3] as u32) << 4) | ((data[4] as u32) >> 4);
    let torq_int = (((data[4] as u32) & 0x0F) << 8) | (data[5] as u32);

    SensorFeedback {
        can_id,
        status_code: status,
        pos: uint_to_float(pos_int, limits.p_min, limits.p_max, 16),
        vel: uint_to_float(vel_int, limits.v_min, limits.v_max, 12),
        torq: uint_to_float(torq_int, limits.t_min, limits.t_max, 12),
        t_mos: data[6] as f32,
        t_rotor: data[7] as f32,
    }
}

pub fn decode_register_value(data: [u8; 8]) -> Result<(u8, [u8; 4])> {
    if !is_register_reply(&data) {
        return Err(MotorError::Protocol(
            "not a register reply frame".to_string(),
        ));
    }
    Ok((data[3], [data[4], data[5], data[6], data[7]]))
}

pub fn encode_enable_cmd() -> [u8; 8] {
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFC]
}

pub fn encode_disable_cmd() -> [u8; 8] {
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFD]
}

pub fn encode_set_zero_cmd() -> [u8; 8] {
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE]
}

pub fn encode_clear_error_cmd() -> [u8; 8] {
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFB]
}

pub fn encode_register_read_cmd(motor_id: u16, rid: u8) -> [u8; 8] {
    [motor_id as u8, (motor_id >> 8) as u8, 0x33, rid, 0, 0, 0, 0]
}

pub fn encode_register_write_cmd(motor_id: u16, rid: u8, data: [u8; 4]) -> [u8; 8] {
    [
        motor_id as u8,
        (motor_id >> 8) as u8,
        0x55,
        rid,
        data[0],
        data[1],
        data[2],
        data[3],
    ]
}

pub fn encode_store_params_cmd(motor_id: u16) -> [u8; 8] {
    [
        motor_id as u8,
        (motor_id >> 8) as u8,
        0xAA,
        0x01,
        0,
        0,
        0,
        0,
    ]
}

pub fn encode_feedback_request_cmd(motor_id: u16) -> [u8; 8] {
    [motor_id as u8, (motor_id >> 8) as u8, 0xCC, 0, 0, 0, 0, 0]
}
