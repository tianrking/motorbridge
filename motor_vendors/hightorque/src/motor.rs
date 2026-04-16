use crate::protocol::{
    decode_read_reply, rad_to_pos_raw, radps_to_vel_raw, torque_nm_to_raw, HightorqueFeedbackState,
};
use motor_core::bus::{CanBus, CanFrame};
use motor_core::device::MotorDevice;
use motor_core::error::{MotorError, Result};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct HightorqueMotor {
    pub motor_id: u16,
    feedback_id: u16,
    model: String,
    bus: Arc<dyn CanBus>,
    state: Mutex<Option<HightorqueFeedbackState>>,
}

impl HightorqueMotor {
    pub(crate) fn new(
        motor_id: u16,
        feedback_id: u16,
        model: &str,
        bus: Arc<dyn CanBus>,
    ) -> Self {
        Self {
            motor_id,
            feedback_id,
            model: model.to_string(),
            bus,
            state: Mutex::new(None),
        }
    }

    pub fn latest_state(&self) -> Option<HightorqueFeedbackState> {
        self.state.lock().ok().and_then(|g| *g)
    }

    pub fn enable(&self) -> Result<()> {
        Err(MotorError::Unsupported(
            "enable is not supported for HighTorque protocol".to_string(),
        ))
    }

    pub fn disable(&self) -> Result<()> {
        self.send_stop()
    }

    pub fn clear_error(&self) -> Result<()> {
        Err(MotorError::InvalidArgument(
            "clear_error is not supported for HighTorque".to_string(),
        ))
    }

    pub fn set_zero_position(&self) -> Result<()> {
        self.send_ext(&[0x40, 0x01, 0x04, 0x64, 0x20, 0x63, 0x0A], 7)?;
        std::thread::sleep(Duration::from_secs(1));
        self.store_parameters()
    }

    pub fn ensure_control_mode(&self, mode: u32, _timeout: Duration) -> Result<()> {
        if (1..=4).contains(&mode) {
            Ok(())
        } else {
            Err(MotorError::InvalidArgument(
                "HighTorque mode must be 1(MIT)/2(POS_VEL)/3(VEL)/4(FORCE_POS)".to_string(),
            ))
        }
    }

    pub fn send_cmd_mit(
        &self,
        target_position: f32,
        target_velocity: f32,
        _stiffness: f32,
        _damping: f32,
        feedforward_torque: f32,
    ) -> Result<()> {
        let pos_raw = rad_to_pos_raw(target_position);
        let vel_raw = radps_to_vel_raw(target_velocity);
        let tqe_raw = torque_nm_to_raw(feedforward_torque);
        self.send_cmd_pos_vel_tqe(pos_raw, vel_raw, tqe_raw)
    }

    pub fn send_cmd_pos_vel(&self, target_position: f32, velocity_limit: f32) -> Result<()> {
        let pos_raw = rad_to_pos_raw(target_position);
        let vel_raw = radps_to_vel_raw(velocity_limit);
        self.send_cmd_pos_vel_tqe(pos_raw, vel_raw, i16::MIN)
    }

    pub fn send_cmd_vel(&self, target_velocity: f32) -> Result<()> {
        let vel_raw = radps_to_vel_raw(target_velocity);
        let mut data = [0x07, 0x07, 0x00, 0x80, 0x20, 0x00, 0x80, 0x00];
        data[4..6].copy_from_slice(&vel_raw.to_le_bytes());
        data[6..8].copy_from_slice(&i16::MIN.to_le_bytes());
        self.send_ext(&data, 8)
    }

    pub fn send_cmd_force_pos(
        &self,
        _target_position: f32,
        _velocity_limit: f32,
        _torque_limit_ratio: f32,
    ) -> Result<()> {
        Err(MotorError::InvalidArgument(
            "send_force_pos is not supported for HighTorque ABI; use send_mit/send_pos_vel"
                .to_string(),
        ))
    }

    pub fn store_parameters(&self) -> Result<()> {
        self.send_ext(&[0x05, 0xB3, 0x02, 0x00, 0x00], 5)
    }

    pub fn request_motor_feedback(&self, timeout: Duration) -> Result<()> {
        self.send_ext(&[0x17, 0x01, 0, 0, 0, 0, 0, 0], 8)?;
        self.wait_status(timeout)
    }

    fn send_stop(&self) -> Result<()> {
        self.send_ext(&[0x01, 0x00, 0x00], 3)
    }

    fn send_cmd_pos_vel_tqe(&self, pos_raw: i16, vel_raw: i16, tqe_raw: i16) -> Result<()> {
        let mut data = [0x07, 0x35, 0, 0, 0, 0, 0, 0];
        data[2..4].copy_from_slice(&vel_raw.to_le_bytes());
        data[4..6].copy_from_slice(&tqe_raw.to_le_bytes());
        data[6..8].copy_from_slice(&pos_raw.to_le_bytes());
        self.send_ext(&data, 8)
    }

    fn send_ext(&self, payload: &[u8], dlc: u8) -> Result<()> {
        let mut data = [0u8; 8];
        data[..payload.len().min(8)].copy_from_slice(&payload[..payload.len().min(8)]);
        self.bus.send(CanFrame {
            arbitration_id: u32::from(0x8000u16 | self.motor_id),
            data,
            dlc: dlc.min(8),
            is_extended: true,
            is_rx: false,
        })
    }

    fn wait_status(&self, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let left = deadline.saturating_duration_since(Instant::now());
            if let Some(frame) = self.bus.recv(left.min(Duration::from_millis(20)))? {
                if let Some(state) = decode_read_reply(frame) {
                    if state.can_id as u16 == self.motor_id {
                        if let Ok(mut g) = self.state.lock() {
                            *g = Some(state);
                        }
                        return Ok(());
                    }
                }
            }
        }
        Err(MotorError::Timeout(format!(
            "HighTorque read timeout on motor id {}",
            self.motor_id
        )))
    }
}

impl MotorDevice for HightorqueMotor {
    fn vendor(&self) -> &'static str {
        "hightorque"
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
        HightorqueMotor::enable(self)
    }

    fn disable(&self) -> Result<()> {
        HightorqueMotor::disable(self)
    }

    fn accepts_frame(&self, frame: &CanFrame) -> bool {
        if !frame.is_rx || frame.dlc < 8 || frame.data[0] != 0x27 || frame.data[1] != 0x01 {
            return false;
        }
        let can_id = if !frame.is_extended && (frame.arbitration_id & 0x00FF) == 0 {
            ((frame.arbitration_id >> 8) & 0x7F) as u16
        } else {
            (frame.arbitration_id & 0x7F) as u16
        };
        can_id == self.motor_id
    }

    fn process_feedback_frame(&self, frame: CanFrame) -> Result<()> {
        if let Some(state) = decode_read_reply(frame) {
            if state.can_id as u16 != self.motor_id {
                return Ok(());
            }
            self.state
                .lock()
                .map_err(|_| MotorError::Io("state lock poisoned".to_string()))?
                .replace(state);
        }
        Ok(())
    }
}
