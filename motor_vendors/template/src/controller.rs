use crate::motor::TemplateMotor;
use motor_core::bus::CanBus;
use motor_core::controller::CoreController;
use motor_core::error::{MotorError, Result};
#[cfg(target_os = "linux")]
use motor_core::socketcan::SocketCanBus;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct TemplateController {
    core: CoreController,
    motors: Mutex<HashMap<u16, Arc<TemplateMotor>>>,
}

impl TemplateController {
    pub fn new(bus: Arc<dyn CanBus>) -> Self {
        Self {
            core: CoreController::new(bus),
            motors: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_socketcan(channel: &str) -> Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let bus: Arc<dyn CanBus> = Arc::new(SocketCanBus::open(channel)?);
            return Ok(Self::new(bus));
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = channel;
            Err(MotorError::InvalidArgument(
                "SocketCAN backend is only supported on Linux".to_string(),
            ))
        }
    }

    pub fn add_motor(
        &self,
        motor_id: u16,
        feedback_id: u16,
        model: &str,
    ) -> Result<Arc<TemplateMotor>> {
        let motor = Arc::new(TemplateMotor::new(
            motor_id,
            feedback_id,
            model,
            self.core.bus(),
        )?);
        let device: Arc<dyn motor_core::device::MotorDevice> = motor.clone();
        self.core.add_device(device)?;
        self.motors
            .lock()
            .map_err(|_| MotorError::Io("motors lock poisoned".to_string()))?
            .insert(motor_id, Arc::clone(&motor));
        Ok(motor)
    }

    pub fn get_motor(&self, motor_id: u16) -> Result<Arc<TemplateMotor>> {
        self.motors
            .lock()
            .map_err(|_| MotorError::Io("motors lock poisoned".to_string()))?
            .get(&motor_id)
            .cloned()
            .ok_or_else(|| MotorError::InvalidArgument(format!("motor {motor_id} not found")))
    }

    pub fn poll_feedback_once(&self) -> Result<()> {
        self.core.poll_feedback_once()
    }

    pub fn enable_all(&self) -> Result<()> {
        self.core.enable_all()
    }

    pub fn disable_all(&self) -> Result<()> {
        self.core.disable_all()
    }

    pub fn shutdown(&self) -> Result<()> {
        self.core.shutdown()
    }
}
