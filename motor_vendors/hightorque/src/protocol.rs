use motor_core::bus::CanFrame;
use std::f32::consts::PI;

const TWO_PI: f32 = PI * 2.0;

#[derive(Debug, Clone, Copy)]
pub struct HightorqueFeedbackState {
    pub can_id: u8,
    pub arbitration_id: u32,
    pub status_code: u8,
    pub pos: f32,
    pub vel: f32,
    pub torq: f32,
    pub t_mos: f32,
    pub t_rotor: f32,
}

pub(crate) fn decode_read_reply(frame: CanFrame) -> Option<HightorqueFeedbackState> {
    if frame.dlc < 8 {
        return None;
    }
    if frame.data[0] != 0x27 || frame.data[1] != 0x01 {
        return None;
    }
    let can_id = if !frame.is_extended && (frame.arbitration_id & 0x00FF) == 0 {
        ((frame.arbitration_id >> 8) & 0x7F) as u8
    } else {
        (frame.arbitration_id & 0x7F) as u8
    };
    let pos_raw = i16::from_le_bytes([frame.data[2], frame.data[3]]);
    let vel_raw = i16::from_le_bytes([frame.data[4], frame.data[5]]);
    let tqe_raw = i16::from_le_bytes([frame.data[6], frame.data[7]]);
    Some(HightorqueFeedbackState {
        can_id,
        arbitration_id: frame.arbitration_id,
        status_code: 0,
        pos: pos_raw as f32 * 0.0001 * TWO_PI,
        vel: vel_raw as f32 * 0.00025 * TWO_PI,
        torq: tqe_raw as f32 * 0.01,
        t_mos: 0.0,
        t_rotor: 0.0,
    })
}

pub(crate) fn rad_to_pos_raw(rad: f32) -> i16 {
    let v = (rad / TWO_PI * 10_000.0).round() as i32;
    v.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

pub(crate) fn radps_to_vel_raw(radps: f32) -> i16 {
    let v = (radps / TWO_PI / 0.00025).round() as i32;
    v.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

pub(crate) fn torque_nm_to_raw(tau_nm: f32) -> i16 {
    let v = (tau_nm * 100.0).round() as i32;
    v.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}
