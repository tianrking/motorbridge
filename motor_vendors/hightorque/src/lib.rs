use motor_core::bus::{CanBus, CanFrame};
use motor_core::error::{MotorError, Result};
#[cfg(target_os = "windows")]
use motor_core::pcan::PcanBus;
#[cfg(target_os = "linux")]
use motor_core::socketcan::SocketCanBus;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

pub struct HightorqueController {
    bus: Arc<dyn CanBus>,
}

impl HightorqueController {
    pub fn new_socketcan(channel: &str) -> Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let bus: Arc<dyn CanBus> = Arc::new(SocketCanBus::open(channel)?);
            return Ok(Self { bus });
        }
        #[cfg(target_os = "windows")]
        {
            let bus: Arc<dyn CanBus> = Arc::new(PcanBus::open(channel)?);
            return Ok(Self { bus });
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = channel;
            Err(MotorError::InvalidArgument(
                "No CAN backend for current platform".to_string(),
            ))
        }
    }

    pub fn add_motor(
        &self,
        motor_id: u16,
        feedback_id: u16,
        model: &str,
    ) -> Result<Arc<HightorqueMotor>> {
        let m = model.trim().to_ascii_lowercase();
        if !(m.is_empty() || m == "hightorque" || m == "ht" || m == "auto" || m == "default") {
            return Err(MotorError::InvalidArgument(format!(
                "unsupported HighTorque model hint: {model}"
            )));
        }
        Ok(Arc::new(HightorqueMotor::new(
            motor_id,
            feedback_id,
            model,
            self.bus.clone(),
        )))
    }

    pub fn poll_feedback_once(&self) -> Result<()> {
        let _ = self.bus.recv(Duration::from_millis(0))?;
        Ok(())
    }

    pub fn enable_all(&self) -> Result<()> {
        Ok(())
    }

    pub fn disable_all(&self) -> Result<()> {
        Ok(())
    }

    pub fn shutdown(&self) -> Result<()> {
        self.bus.shutdown()
    }

    pub fn close_bus(&self) -> Result<()> {
        self.bus.shutdown()
    }
}

pub struct HightorqueMotor {
    pub motor_id: u16,
    bus: Arc<dyn CanBus>,
    state: Mutex<Option<HightorqueFeedbackState>>,
}

impl HightorqueMotor {
    fn new(motor_id: u16, _feedback_id: u16, _model: &str, bus: Arc<dyn CanBus>) -> Self {
        Self {
            motor_id,
            bus,
            state: Mutex::new(None),
        }
    }

    pub fn latest_state(&self) -> Option<HightorqueFeedbackState> {
        self.state.lock().ok().and_then(|g| *g)
    }

    pub fn enable(&self) -> Result<()> {
        Ok(())
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

fn decode_read_reply(frame: CanFrame) -> Option<HightorqueFeedbackState> {
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

fn rad_to_pos_raw(rad: f32) -> i16 {
    let v = (rad / TWO_PI * 10_000.0).round() as i32;
    v.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn radps_to_vel_raw(radps: f32) -> i16 {
    let v = (radps / TWO_PI / 0.00025).round() as i32;
    v.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn torque_nm_to_raw(tau_nm: f32) -> i16 {
    let v = (tau_nm * 100.0).round() as i32;
    v.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}
