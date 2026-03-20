use crate::protocol::{
    decode_frame, encode_current_setpoint, encode_position_absolute_setpoint,
    encode_single_command, encode_velocity_setpoint, request_arbitration_id,
    response_arbitration_id, Command, DecodedFrame,
};
use motor_core::bus::{CanBus, CanFrame};
use motor_core::device::MotorDevice;
use motor_core::error::{MotorError, Result};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum ControlMode {
    Current = 1,
    Velocity = 2,
    Position = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct MyActuatorFeedbackState {
    pub arbitration_id: u32,
    pub command: u8,
    pub temperature_c: i8,
    pub current_a: f32,
    pub speed_dps: f32,
    pub shaft_angle_deg: f32,
}

pub struct MyActuatorMotor {
    pub motor_id: u16,
    pub feedback_id: u16,
    pub model: String,
    bus: Arc<dyn CanBus>,
    state: Mutex<Option<MyActuatorFeedbackState>>,
    multi_turn_angle_deg: Mutex<Option<f32>>,
    version_date: Mutex<Option<u32>>,
    control_mode: Mutex<Option<u8>>,
}

impl MyActuatorMotor {
    pub fn new(motor_id: u16, feedback_id: u16, model: &str, bus: Arc<dyn CanBus>) -> Result<Self> {
        if motor_id == 0 || motor_id > 32 {
            return Err(MotorError::InvalidArgument(format!(
                "MyActuator motor_id out of range [1,32]: {motor_id}"
            )));
        }
        let default_feedback_id = response_arbitration_id(motor_id) as u16;
        let effective_feedback_id = if feedback_id == 0 {
            default_feedback_id
        } else {
            feedback_id
        };
        Ok(Self {
            motor_id,
            feedback_id: effective_feedback_id,
            model: model.to_string(),
            bus,
            state: Mutex::new(None),
            multi_turn_angle_deg: Mutex::new(None),
            version_date: Mutex::new(None),
            control_mode: Mutex::new(None),
        })
    }

    fn send_raw(&self, data: [u8; 8]) -> Result<()> {
        self.bus.send(CanFrame {
            arbitration_id: request_arbitration_id(self.motor_id),
            data,
            dlc: 8,
            is_extended: false,
            is_rx: false,
        })
    }

    pub fn latest_state(&self) -> Option<MyActuatorFeedbackState> {
        self.state.lock().ok().and_then(|s| *s)
    }

    pub fn latest_version_date(&self) -> Option<u32> {
        self.version_date.lock().ok().and_then(|v| *v)
    }

    pub fn latest_multi_turn_angle_deg(&self) -> Option<f32> {
        self.multi_turn_angle_deg.lock().ok().and_then(|a| *a)
    }

    pub fn latest_control_mode(&self) -> Option<u8> {
        self.control_mode.lock().ok().and_then(|m| *m)
    }

    pub fn await_version_date(&self, timeout: Duration) -> Result<u32> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(v) = self.latest_version_date() {
                return Ok(v);
            }
            if Instant::now() >= deadline {
                return Err(MotorError::Timeout(format!(
                    "MyActuator motor {} version query timed out",
                    self.motor_id
                )));
            }
            std::thread::sleep(Duration::from_millis(8));
        }
    }

    pub fn release_brake(&self) -> Result<()> {
        self.send_raw(encode_single_command(Command::ReleaseBrake))
    }

    pub fn stop_motor(&self) -> Result<()> {
        self.send_raw(encode_single_command(Command::StopMotor))
    }

    pub fn shutdown_motor(&self) -> Result<()> {
        self.send_raw(encode_single_command(Command::ShutdownMotor))
    }

    pub fn request_status(&self) -> Result<()> {
        self.send_raw(encode_single_command(Command::ReadMotorStatus2))
    }

    pub fn request_multi_turn_angle(&self) -> Result<()> {
        self.send_raw(encode_single_command(Command::ReadMultiTurnAngle))
    }

    pub fn request_version_date(&self) -> Result<()> {
        self.send_raw(encode_single_command(Command::ReadVersionDate))
    }

    pub fn request_control_mode(&self) -> Result<()> {
        self.send_raw(encode_single_command(Command::ReadSystemOperatingMode))
    }

    pub fn set_current_position_as_zero(&self) -> Result<()> {
        self.send_raw(encode_single_command(Command::WriteCurrentPositionAsZero))
    }

    pub fn send_current_setpoint(&self, current_a: f32) -> Result<()> {
        self.send_raw(encode_current_setpoint(current_a))
    }

    pub fn send_velocity_setpoint(&self, speed_dps: f32) -> Result<()> {
        self.send_raw(encode_velocity_setpoint(speed_dps))
    }

    pub fn send_position_absolute_setpoint(
        &self,
        position_deg: f32,
        max_speed_dps: f32,
    ) -> Result<()> {
        self.send_raw(encode_position_absolute_setpoint(
            position_deg,
            max_speed_dps,
        ))
    }

    fn process_feedback_frame_impl(&self, frame: CanFrame) -> Result<()> {
        match decode_frame(frame.data)? {
            DecodedFrame::Feedback(fb) => {
                self.state
                    .lock()
                    .map_err(|_| MotorError::Io("state lock poisoned".to_string()))?
                    .replace(MyActuatorFeedbackState {
                        arbitration_id: frame.arbitration_id,
                        command: fb.command,
                        temperature_c: fb.temperature_c,
                        current_a: fb.current_a,
                        speed_dps: fb.speed_dps,
                        shaft_angle_deg: fb.shaft_angle_deg,
                    });
            }
            DecodedFrame::MultiTurnAngleDeg(angle_deg) => {
                self.multi_turn_angle_deg
                    .lock()
                    .map_err(|_| MotorError::Io("multi-turn-angle lock poisoned".to_string()))?
                    .replace(angle_deg);
            }
            DecodedFrame::VersionDate(version) => {
                self.version_date
                    .lock()
                    .map_err(|_| MotorError::Io("version lock poisoned".to_string()))?
                    .replace(version);
            }
            DecodedFrame::ControlMode(mode) => {
                self.control_mode
                    .lock()
                    .map_err(|_| MotorError::Io("mode lock poisoned".to_string()))?
                    .replace(mode);
            }
            DecodedFrame::Ack(_) => {}
        }
        Ok(())
    }
}

impl MotorDevice for MyActuatorMotor {
    fn vendor(&self) -> &'static str {
        "myactuator"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn motor_id(&self) -> u16 {
        self.motor_id
    }

    fn feedback_id(&self) -> u16 {
        self.feedback_id
    }

    fn enable(&self) -> Result<()> {
        self.release_brake()
    }

    fn disable(&self) -> Result<()> {
        self.shutdown_motor()
    }

    fn accepts_frame(&self, frame: &CanFrame) -> bool {
        !frame.is_extended && frame.arbitration_id == u32::from(self.feedback_id)
    }

    fn process_feedback_frame(&self, frame: CanFrame) -> Result<()> {
        self.process_feedback_frame_impl(frame)
    }
}
