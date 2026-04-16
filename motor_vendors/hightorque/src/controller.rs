use crate::motor::HightorqueMotor;
use motor_core::bus::{CanBus, open_socketcan};
use motor_core::error::{MotorError, Result};
use motor_core::vendor_controller::VendorController;
use std::sync::Arc;

pub struct HightorqueController {
    controller: VendorController<HightorqueMotor>,
}

impl HightorqueController {
    pub fn new(bus: Arc<dyn CanBus>) -> Self {
        Self {
            controller: VendorController::new(bus),
        }
    }

    pub fn new_socketcan(channel: &str) -> Result<Self> {
        Ok(Self::new(open_socketcan(channel)?))
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
        self.controller.add_motor_with(motor_id, |bus| {
            Ok(HightorqueMotor::new(motor_id, feedback_id, model, bus))
        })
    }

    pub fn poll_feedback_once(&self) -> Result<()> {
        self.controller.poll_feedback_once()
    }

    pub fn enable_all(&self) -> Result<()> {
        self.controller.enable_all()
    }

    pub fn disable_all(&self) -> Result<()> {
        self.controller.disable_all()
    }

    pub fn shutdown(&self) -> Result<()> {
        self.controller.shutdown()
    }

    pub fn close_bus(&self) -> Result<()> {
        self.controller.close_bus()
    }
}
