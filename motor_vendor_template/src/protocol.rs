use motor_core::error::{MotorError, Result};

#[derive(Debug, Clone, Copy)]
pub struct DecodedFeedback {
    pub logical_id: u8,
    pub status_code: u8,
    pub pos: f32,
    pub vel: f32,
    pub torq: f32,
    pub temp_a: f32,
    pub temp_b: f32,
}

pub fn encode_enable_cmd() -> [u8; 8] {
    // TODO: Replace with vendor-specific enable frame.
    [0xFF; 8]
}

pub fn encode_disable_cmd() -> [u8; 8] {
    // TODO: Replace with vendor-specific disable frame.
    [0x00; 8]
}

pub fn decode_feedback_frame(_data: [u8; 8]) -> Result<DecodedFeedback> {
    Err(MotorError::Unsupported(
        "decode_feedback_frame is not implemented for template vendor".to_string(),
    ))
}
