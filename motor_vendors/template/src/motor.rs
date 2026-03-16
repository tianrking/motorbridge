use crate::protocol;
use motor_core::bus::{CanBus, CanFrame};
use motor_core::device::MotorDevice;
use motor_core::error::{MotorError, Result};
use motor_core::model::{ModelCatalog, MotorModelSpec, StaticModelCatalog};
use std::sync::{Arc, Mutex};

const TEMPLATE_MODELS: &[MotorModelSpec] = &[
    // TODO: Replace with actual vendor models.
    MotorModelSpec {
        vendor: "template_vendor",
        model: "model_a",
        pmax: 12.5,
        vmax: 20.0,
        tmax: 10.0,
    },
];

const TEMPLATE_CATALOG: StaticModelCatalog = StaticModelCatalog {
    vendor_name: "template_vendor",
    models: TEMPLATE_MODELS,
};

#[derive(Debug, Clone, Copy)]
pub struct TemplateMotorState {
    pub arbitration_id: u16,
    pub status_code: u8,
    pub pos: f32,
    pub vel: f32,
    pub torq: f32,
    pub temp_a: f32,
    pub temp_b: f32,
}

pub struct TemplateMotor {
    pub motor_id: u16,
    pub feedback_id: u16,
    pub model: String,
    bus: Arc<dyn CanBus>,
    state: Mutex<Option<TemplateMotorState>>,
}

impl TemplateMotor {
    pub fn new(motor_id: u16, feedback_id: u16, model: &str, bus: Arc<dyn CanBus>) -> Result<Self> {
        if TEMPLATE_CATALOG.get(model).is_none() {
            return Err(MotorError::InvalidArgument(format!(
                "unknown template vendor model: {model}"
            )));
        }
        Ok(Self {
            motor_id,
            feedback_id,
            model: model.to_string(),
            bus,
            state: Mutex::new(None),
        })
    }

    pub fn latest_state(&self) -> Option<TemplateMotorState> {
        self.state.lock().ok().and_then(|s| *s)
    }

    fn send_raw(&self, arbitration_id: u16, data: [u8; 8]) -> Result<()> {
        self.bus.send(CanFrame {
            arbitration_id,
            data,
            is_rx: false,
        })
    }

    fn process_feedback_frame_impl(&self, frame: CanFrame) -> Result<()> {
        let decoded = protocol::decode_feedback_frame(frame.data)?;
        self.state
            .lock()
            .map_err(|_| MotorError::Io("state lock poisoned".to_string()))?
            .replace(TemplateMotorState {
                arbitration_id: frame.arbitration_id,
                status_code: decoded.status_code,
                pos: decoded.pos,
                vel: decoded.vel,
                torq: decoded.torq,
                temp_a: decoded.temp_a,
                temp_b: decoded.temp_b,
            });
        Ok(())
    }
}

impl MotorDevice for TemplateMotor {
    fn vendor(&self) -> &'static str {
        "template_vendor"
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

    fn feedback_logical_id(&self) -> u8 {
        // TODO: Replace if your protocol routes differently.
        (self.motor_id & 0x0F) as u8
    }

    fn enable(&self) -> Result<()> {
        self.send_raw(self.motor_id, protocol::encode_enable_cmd())
    }

    fn disable(&self) -> Result<()> {
        self.send_raw(self.motor_id, protocol::encode_disable_cmd())
    }

    fn process_feedback_frame(&self, frame: CanFrame) -> Result<()> {
        self.process_feedback_frame_impl(frame)
    }
}
