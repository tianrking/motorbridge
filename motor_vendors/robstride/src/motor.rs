use crate::protocol::{
    build_ext_id, decode_ping_reply, decode_read_parameter_value, decode_status_frame,
    encode_mit_command, encode_parameter_read, encode_parameter_value, encode_parameter_write,
    ext_id_parts, CommunicationType, PingReply,
};
use crate::registers::{parameter_info, ParameterDataType, ParameterId};
use motor_core::bus::{CanBus, CanFrame};
use motor_core::device::MotorDevice;
use motor_core::error::{MotorError, Result};
use motor_core::model::{ModelCatalog, MotorModelSpec, PvTLimits, StaticModelCatalog};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const ROBSTRIDE_MODELS: &[MotorModelSpec] = &[
    MotorModelSpec {
        vendor: "robstride",
        model: "rs-00",
        pmax: 4.0 * std::f32::consts::PI,
        vmax: 50.0,
        tmax: 17.0,
    },
    MotorModelSpec {
        vendor: "robstride",
        model: "rs-01",
        pmax: 4.0 * std::f32::consts::PI,
        vmax: 44.0,
        tmax: 17.0,
    },
    MotorModelSpec {
        vendor: "robstride",
        model: "rs-02",
        pmax: 4.0 * std::f32::consts::PI,
        vmax: 44.0,
        tmax: 17.0,
    },
    MotorModelSpec {
        vendor: "robstride",
        model: "rs-03",
        pmax: 4.0 * std::f32::consts::PI,
        vmax: 50.0,
        tmax: 60.0,
    },
    MotorModelSpec {
        vendor: "robstride",
        model: "rs-04",
        pmax: 4.0 * std::f32::consts::PI,
        vmax: 15.0,
        tmax: 120.0,
    },
    MotorModelSpec {
        vendor: "robstride",
        model: "rs-05",
        pmax: 4.0 * std::f32::consts::PI,
        vmax: 33.0,
        tmax: 17.0,
    },
    MotorModelSpec {
        vendor: "robstride",
        model: "rs-06",
        pmax: 4.0 * std::f32::consts::PI,
        vmax: 20.0,
        tmax: 60.0,
    },
];

const ROBSTRIDE_CATALOG: StaticModelCatalog = StaticModelCatalog {
    vendor_name: "robstride",
    models: ROBSTRIDE_MODELS,
};

pub fn model_limits(model: &str) -> Option<(f32, f32, f32)> {
    ROBSTRIDE_CATALOG
        .get(model)
        .map(|spec| (spec.pmax, spec.vmax, spec.tmax))
}

#[derive(Debug, Clone, Copy)]
pub enum ControlMode {
    Mit = 0,
    Position = 1,
    Velocity = 2,
}

#[derive(Debug, Clone, Copy)]
pub enum ParameterValue {
    I8(i8),
    U8(u8),
    U16(u16),
    U32(u32),
    F32(f32),
}

#[derive(Debug, Clone, Copy)]
pub struct MotorFeedbackState {
    pub arbitration_id: u32,
    pub device_id: u8,
    pub position: f32,
    pub velocity: f32,
    pub torque: f32,
    pub temperature_c: f32,
    pub uncalibrated: bool,
    pub stall: bool,
    pub magnetic_encoder_fault: bool,
    pub overtemperature: bool,
    pub overcurrent: bool,
    pub undervoltage: bool,
}

pub struct RobstrideMotor {
    pub motor_id: u16,
    pub feedback_id: u16,
    pub model: String,
    bus: Arc<dyn CanBus>,
    limits: PvTLimits,
    kp_max: f32,
    kd_max: f32,
    state: Mutex<Option<MotorFeedbackState>>,
    registers: Mutex<HashMap<u16, ParameterValue>>,
    ping_reply: Mutex<Option<PingReply>>,
    pending_param: Mutex<Option<u16>>,
}

impl RobstrideMotor {
    pub fn new(motor_id: u16, feedback_id: u16, model: &str, bus: Arc<dyn CanBus>) -> Result<Self> {
        let spec = ROBSTRIDE_CATALOG.get(model).ok_or_else(|| {
            MotorError::InvalidArgument(format!("unknown RobStride model: {model}"))
        })?;
        let (kp_max, kd_max) = match model {
            "rs-00" | "rs-01" | "rs-02" | "rs-05" => (500.0, 5.0),
            "rs-03" | "rs-04" | "rs-06" => (5000.0, 100.0),
            _ => (500.0, 5.0),
        };
        Ok(Self {
            motor_id,
            feedback_id,
            model: model.to_string(),
            bus,
            limits: PvTLimits::from_spec(spec),
            kp_max,
            kd_max,
            state: Mutex::new(None),
            registers: Mutex::new(HashMap::new()),
            ping_reply: Mutex::new(None),
            pending_param: Mutex::new(None),
        })
    }

    fn device_id_u8(&self) -> Result<u8> {
        u8::try_from(self.motor_id).map_err(|_| {
            MotorError::InvalidArgument(format!("motor_id {} out of u8 range", self.motor_id))
        })
    }

    fn host_id_u16(&self) -> u16 {
        match self.feedback_id {
            1..=0xFF => self.feedback_id,
            _ => 0x00FF,
        }
    }

    fn host_id_u8(&self) -> u8 {
        self.host_id_u16() as u8
    }

    fn host_id_candidates(&self) -> Vec<u16> {
        let mut cands = Vec::with_capacity(3);
        let push_unique = |v: u16, out: &mut Vec<u16>| {
            if (1..=0xFF).contains(&v) && !out.contains(&v) {
                out.push(v);
            }
        };
        push_unique(self.host_id_u16(), &mut cands);
        push_unique(0x00FF, &mut cands);
        push_unique(0x00FE, &mut cands);
        cands
    }

    fn send_ext(&self, comm_type: u32, extra_data: u16, data: [u8; 8], dlc: u8) -> Result<()> {
        self.bus.send(CanFrame {
            arbitration_id: build_ext_id(comm_type, extra_data, self.device_id_u8()?),
            data,
            dlc,
            is_extended: true,
            is_rx: false,
        })
    }

    pub fn ping(&self, timeout: Duration) -> Result<PingReply> {
        let cands = self.host_id_candidates();
        let per_try = {
            let ms = (timeout.as_millis() as u64).max(60);
            Duration::from_millis((ms / cands.len().max(1) as u64).max(60))
        };
        for host in cands {
            self.ping_reply
                .lock()
                .map_err(|_| MotorError::Io("ping reply lock poisoned".to_string()))?
                .take();
            self.send_ext(CommunicationType::GET_DEVICE_ID, host, [0u8; 8], 8)?;
            let deadline = Instant::now() + per_try;
            loop {
                if let Some(reply) = *self
                    .ping_reply
                    .lock()
                    .map_err(|_| MotorError::Io("ping reply lock poisoned".to_string()))?
                {
                    return Ok(reply);
                }
                if Instant::now() >= deadline {
                    break;
                }
                std::thread::sleep(Duration::from_millis(8));
            }
        }
        Err(MotorError::Timeout(format!(
            "ping {} timed out",
            self.motor_id
        )))
    }

    pub fn set_mode(&self, mode: ControlMode) -> Result<()> {
        self.write_parameter(ParameterId::Mode as u16, ParameterValue::I8(mode as i8))
    }

    pub fn set_zero_position(&self) -> Result<()> {
        self.send_ext(
            CommunicationType::SET_ZERO_POSITION,
            self.host_id_u16(),
            [0u8; 8],
            8,
        )
    }

    pub fn save_parameters(&self) -> Result<()> {
        self.send_ext(
            CommunicationType::SAVE_PARAMETERS,
            self.host_id_u16(),
            [0u8; 8],
            8,
        )
    }

    pub fn set_device_id(&self, new_id: u8) -> Result<()> {
        self.send_ext(CommunicationType::SET_DEVICE_ID, new_id as u16, [0u8; 8], 8)
    }

    pub fn enable(&self) -> Result<()> {
        self.send_ext(CommunicationType::ENABLE, self.host_id_u16(), [0u8; 8], 8)
    }

    pub fn disable(&self) -> Result<()> {
        self.send_ext(CommunicationType::DISABLE, self.host_id_u16(), [0u8; 8], 8)
    }

    pub fn send_cmd_mit(
        &self,
        target_position: f32,
        target_velocity: f32,
        stiffness: f32,
        damping: f32,
        feedforward_torque: f32,
    ) -> Result<()> {
        let (extra_data, data) = encode_mit_command(
            target_position,
            target_velocity,
            stiffness,
            damping,
            feedforward_torque,
            self.limits.p_max,
            self.limits.v_max,
            self.limits.t_max,
            self.kp_max,
            self.kd_max,
        );
        self.send_ext(CommunicationType::OPERATION_CONTROL, extra_data, data, 8)
    }

    pub fn set_velocity_target(&self, velocity: f32) -> Result<()> {
        self.write_parameter(
            ParameterId::VelocityTarget as u16,
            ParameterValue::F32(velocity),
        )
    }

    pub fn write_parameter(&self, param_id: u16, value: ParameterValue) -> Result<()> {
        let raw = encode_parameter_value(param_id, value)?;
        let data = encode_parameter_write(param_id, raw);
        self.send_ext(
            CommunicationType::WRITE_PARAMETER,
            self.host_id_u16(),
            data,
            8,
        )
    }

    pub fn request_parameter(&self, param_id: u16) -> Result<()> {
        self.registers
            .lock()
            .map_err(|_| MotorError::Io("register lock poisoned".to_string()))?
            .remove(&param_id);
        self.pending_param
            .lock()
            .map_err(|_| MotorError::Io("pending param lock poisoned".to_string()))?
            .replace(param_id);
        let data = encode_parameter_read(param_id);
        self.send_ext(
            CommunicationType::READ_PARAMETER,
            self.host_id_u16(),
            data,
            8,
        )
    }

    pub fn get_parameter(&self, param_id: u16, timeout: Duration) -> Result<ParameterValue> {
        let cands = self.host_id_candidates();
        let per_try = {
            let ms = (timeout.as_millis() as u64).max(90);
            Duration::from_millis((ms / cands.len().max(1) as u64).max(90))
        };

        for host in cands {
            self.registers
                .lock()
                .map_err(|_| MotorError::Io("register lock poisoned".to_string()))?
                .remove(&param_id);
            self.pending_param
                .lock()
                .map_err(|_| MotorError::Io("pending param lock poisoned".to_string()))?
                .replace(param_id);
            let data = encode_parameter_read(param_id);
            self.send_ext(CommunicationType::READ_PARAMETER, host, data, 8)?;

            let deadline = Instant::now() + per_try;
            loop {
                if let Some(value) = self
                    .registers
                    .lock()
                    .map_err(|_| MotorError::Io("register lock poisoned".to_string()))?
                    .get(&param_id)
                    .copied()
                {
                    return Ok(value);
                }
                if Instant::now() >= deadline {
                    break;
                }
                std::thread::sleep(Duration::from_millis(8));
            }
        }
        Err(MotorError::Timeout(format!(
            "parameter 0x{param_id:04X} not received within {:?}",
            timeout
        )))
    }

    pub fn get_parameter_f32(&self, param_id: u16, timeout: Duration) -> Result<f32> {
        match self.get_parameter(param_id, timeout)? {
            ParameterValue::F32(v) => Ok(v),
            _ => Err(MotorError::Protocol(format!(
                "parameter 0x{param_id:04X} is not f32"
            ))),
        }
    }

    pub fn get_parameter_i8(&self, param_id: u16, timeout: Duration) -> Result<i8> {
        match self.get_parameter(param_id, timeout)? {
            ParameterValue::I8(v) => Ok(v),
            _ => Err(MotorError::Protocol(format!(
                "parameter 0x{param_id:04X} is not i8"
            ))),
        }
    }

    pub fn latest_state(&self) -> Option<MotorFeedbackState> {
        self.state.lock().ok().and_then(|s| *s)
    }

    fn process_feedback_frame_impl(&self, frame: CanFrame) -> Result<()> {
        let (comm_type, extra_data, _) = ext_id_parts(frame.arbitration_id);
        match comm_type {
            CommunicationType::GET_DEVICE_ID => {
                let reply = decode_ping_reply(frame.arbitration_id, frame.data)?;
                self.ping_reply
                    .lock()
                    .map_err(|_| MotorError::Io("ping reply lock poisoned".to_string()))?
                    .replace(reply);
                Ok(())
            }
            CommunicationType::READ_PARAMETER => {
                let param_id = self
                    .pending_param
                    .lock()
                    .map_err(|_| MotorError::Io("pending param lock poisoned".to_string()))?
                    .take()
                    .unwrap_or_else(|| u16::from_le_bytes([frame.data[0], frame.data[1]]));
                let raw = decode_read_parameter_value(param_id, frame.data)?;
                let info = parameter_info(param_id).ok_or_else(|| {
                    MotorError::Protocol(format!("unknown RobStride parameter 0x{param_id:04X}"))
                })?;
                let value = match info.data_type {
                    ParameterDataType::Int8 => ParameterValue::I8(raw[0] as i8),
                    ParameterDataType::UInt8 => ParameterValue::U8(raw[0]),
                    ParameterDataType::UInt16 => {
                        ParameterValue::U16(u16::from_le_bytes([raw[0], raw[1]]))
                    }
                    ParameterDataType::UInt32 => ParameterValue::U32(u32::from_le_bytes(raw)),
                    ParameterDataType::Float32 => ParameterValue::F32(f32::from_le_bytes(raw)),
                };
                self.registers
                    .lock()
                    .map_err(|_| MotorError::Io("register lock poisoned".to_string()))?
                    .insert(param_id, value);
                Ok(())
            }
            CommunicationType::OPERATION_STATUS | CommunicationType::FAULT_REPORT => {
                let status = decode_status_frame(
                    extra_data,
                    frame.data,
                    self.limits.p_max,
                    self.limits.v_max,
                    self.limits.t_max,
                );
                self.state
                    .lock()
                    .map_err(|_| MotorError::Io("state lock poisoned".to_string()))?
                    .replace(MotorFeedbackState {
                        arbitration_id: frame.arbitration_id,
                        device_id: status.flags.device_id,
                        position: status.position,
                        velocity: status.velocity,
                        torque: status.torque,
                        temperature_c: status.temperature_c,
                        uncalibrated: status.flags.uncalibrated,
                        stall: status.flags.stall,
                        magnetic_encoder_fault: status.flags.magnetic_encoder_fault,
                        overtemperature: status.flags.overtemperature,
                        overcurrent: status.flags.overcurrent,
                        undervoltage: status.flags.undervoltage,
                    });
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl MotorDevice for RobstrideMotor {
    fn vendor(&self) -> &'static str {
        "robstride"
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
        RobstrideMotor::enable(self)
    }

    fn disable(&self) -> Result<()> {
        RobstrideMotor::disable(self)
    }

    fn accepts_frame(&self, frame: &CanFrame) -> bool {
        if !frame.is_extended {
            return false;
        }
        let (comm_type, extra_data, responder_id) = ext_id_parts(frame.arbitration_id);
        match comm_type {
            CommunicationType::GET_DEVICE_ID => (extra_data & 0xFF) == self.motor_id,
            CommunicationType::READ_PARAMETER
            | CommunicationType::OPERATION_STATUS
            | CommunicationType::FAULT_REPORT => {
                (extra_data & 0xFF) == self.motor_id || responder_id == self.host_id_u8()
            }
            _ => false,
        }
    }

    fn process_feedback_frame(&self, frame: CanFrame) -> Result<()> {
        self.process_feedback_frame_impl(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct SilentBus {
        sent: Mutex<Vec<CanFrame>>,
    }

    impl SilentBus {
        fn new() -> Self {
            Self {
                sent: Mutex::new(Vec::new()),
            }
        }
    }

    impl CanBus for SilentBus {
        fn send(&self, frame: CanFrame) -> Result<()> {
            self.sent
                .lock()
                .map_err(|_| MotorError::Io("sent lock poisoned".to_string()))?
                .push(frame);
            Ok(())
        }

        fn recv(&self, _timeout: Duration) -> Result<Option<CanFrame>> {
            Ok(None)
        }

        fn shutdown(&self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn get_parameter_times_out_when_no_reply_arrives() {
        let bus: Arc<dyn CanBus> = Arc::new(SilentBus::new());
        let motor = RobstrideMotor::new(127, 0xFF, "rs-00", bus).expect("create motor");
        let err = motor
            .get_parameter(0x7019, Duration::from_millis(5))
            .expect_err("timeout expected");
        assert!(matches!(err, MotorError::Timeout(_)));
    }
}
