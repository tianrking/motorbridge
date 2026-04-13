use crate::motor::HightorqueMotor;
use motor_core::bus::CanBus;
use motor_core::error::{MotorError, Result};
#[cfg(any(target_os = "windows", target_os = "macos"))]
use motor_core::pcan::PcanBus;
#[cfg(target_os = "linux")]
use motor_core::socketcan::SocketCanBus;
use std::sync::Arc;
use std::time::Duration;

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
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            let bus: Arc<dyn CanBus> = Arc::new(PcanBus::open(channel)?);
            return Ok(Self { bus });
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
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
