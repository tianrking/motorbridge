use motor_core::error::Result;

pub const REQUEST_BASE_ID: u32 = 0x140;
pub const RESPONSE_BASE_ID: u32 = 0x240;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    ShutdownMotor = 0x80,
    StopMotor = 0x81,
    TorqueClosedLoopControl = 0xA1,
    SpeedClosedLoopControl = 0xA2,
    AbsolutePositionClosedLoopControl = 0xA4,
    ReadMotorStatus2 = 0x9C,
    ReadSystemOperatingMode = 0x70,
    ReleaseBrake = 0x77,
    ReadVersionDate = 0xB2,
}

#[derive(Debug, Clone, Copy)]
pub struct DecodedFeedback {
    pub command: u8,
    pub temperature_c: i8,
    pub current_a: f32,
    pub speed_dps: f32,
    pub shaft_angle_deg: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum DecodedFrame {
    Feedback(DecodedFeedback),
    VersionDate(u32),
    ControlMode(u8),
    Ack(u8),
}

pub fn request_arbitration_id(motor_id: u16) -> u32 {
    REQUEST_BASE_ID + u32::from(motor_id)
}

pub fn response_arbitration_id(motor_id: u16) -> u32 {
    RESPONSE_BASE_ID + u32::from(motor_id)
}

fn clamp_i16(v: i32) -> i16 {
    v.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

fn clamp_i32(v: i64) -> i32 {
    v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

pub fn encode_single_command(cmd: Command) -> [u8; 8] {
    [cmd as u8, 0, 0, 0, 0, 0, 0, 0]
}

pub fn encode_current_setpoint(current_a: f32) -> [u8; 8] {
    let mut data = encode_single_command(Command::TorqueClosedLoopControl);
    let scaled = clamp_i16((current_a / 0.01).round() as i32).to_le_bytes();
    data[4] = scaled[0];
    data[5] = scaled[1];
    data
}

pub fn encode_velocity_setpoint(speed_dps: f32) -> [u8; 8] {
    let mut data = encode_single_command(Command::SpeedClosedLoopControl);
    let scaled = clamp_i32((speed_dps as f64 * 100.0).round() as i64).to_le_bytes();
    data[4] = scaled[0];
    data[5] = scaled[1];
    data[6] = scaled[2];
    data[7] = scaled[3];
    data
}

pub fn encode_position_absolute_setpoint(position_deg: f32, max_speed_dps: f32) -> [u8; 8] {
    let mut data = encode_single_command(Command::AbsolutePositionClosedLoopControl);
    let max_speed = (max_speed_dps.round() as i32).clamp(0, i32::from(u16::MAX)) as u16;
    let max_speed_le = max_speed.to_le_bytes();
    data[2] = max_speed_le[0];
    data[3] = max_speed_le[1];

    let pos = clamp_i32((position_deg as f64 * 100.0).round() as i64).to_le_bytes();
    data[4] = pos[0];
    data[5] = pos[1];
    data[6] = pos[2];
    data[7] = pos[3];
    data
}

pub fn decode_frame(data: [u8; 8]) -> Result<DecodedFrame> {
    match data[0] {
        x if x == Command::ReadVersionDate as u8 => {
            Ok(DecodedFrame::VersionDate(u32::from_le_bytes([
                data[4], data[5], data[6], data[7],
            ])))
        }
        x if x == Command::ReadSystemOperatingMode as u8 => Ok(DecodedFrame::ControlMode(data[7])),
        x if x == Command::ReadMotorStatus2 as u8
            || x == Command::TorqueClosedLoopControl as u8
            || x == Command::SpeedClosedLoopControl as u8
            || x == Command::AbsolutePositionClosedLoopControl as u8 =>
        {
            let feedback = DecodedFeedback {
                command: data[0],
                temperature_c: data[1] as i8,
                current_a: f32::from(i16::from_le_bytes([data[2], data[3]])) * 0.01,
                speed_dps: f32::from(i16::from_le_bytes([data[4], data[5]])),
                shaft_angle_deg: f32::from(i16::from_le_bytes([data[6], data[7]])),
            };
            Ok(DecodedFrame::Feedback(feedback))
        }
        cmd => Ok(DecodedFrame::Ack(cmd)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_position_absolute_packs_speed_and_position() {
        let data = encode_position_absolute_setpoint(180.25, 500.0);
        assert_eq!(data[0], Command::AbsolutePositionClosedLoopControl as u8);
        assert_eq!(u16::from_le_bytes([data[2], data[3]]), 500);
        assert_eq!(
            i32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            18_025
        );
    }

    #[test]
    fn decode_feedback_status2_layout() {
        let frame = [
            Command::ReadMotorStatus2 as u8,
            25,
            0x10,
            0x27,
            0x34,
            0x12,
            0xFE,
            0xFF,
        ];
        let decoded = decode_frame(frame).expect("decode");
        match decoded {
            DecodedFrame::Feedback(fb) => {
                assert_eq!(fb.temperature_c, 25);
                assert!((fb.current_a - 100.0).abs() < 1e-6);
                assert_eq!(fb.speed_dps, 0x1234 as f32);
                assert_eq!(fb.shaft_angle_deg, -2.0);
            }
            _ => panic!("expected feedback"),
        }
    }
}
