use crate::motor::DamiaoMotor;
use motor_core::bus::CanBus;
use motor_core::controller::CoreController;
use motor_core::dm_serial::DmSerialBus;
use motor_core::error::{MotorError, Result};
#[cfg(target_os = "windows")]
use motor_core::pcan::PcanBus;
#[cfg(target_os = "linux")]
use motor_core::socketcan::SocketCanBus;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct DamiaoController {
    core: CoreController,
    motors: Mutex<HashMap<u16, Arc<DamiaoMotor>>>,
}

impl DamiaoController {
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
        #[cfg(target_os = "windows")]
        {
            let bus: Arc<dyn CanBus> = Arc::new(PcanBus::open(channel)?);
            return Ok(Self::new(bus));
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = channel;
            Err(MotorError::InvalidArgument(
                "No CAN backend for current platform".to_string(),
            ))
        }
    }

    pub fn new_dm_serial(port: &str, baud: u32) -> Result<Self> {
        let bus: Arc<dyn CanBus> = Arc::new(DmSerialBus::open(port, baud)?);
        Ok(Self::new(bus))
    }

    pub fn add_motor(
        &self,
        motor_id: u16,
        feedback_id: u16,
        model: &str,
    ) -> Result<Arc<DamiaoMotor>> {
        let motor = Arc::new(DamiaoMotor::new(
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

    pub fn get_motor(&self, motor_id: u16) -> Result<Arc<DamiaoMotor>> {
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

    pub fn close_bus(&self) -> Result<()> {
        self.core.close_bus()
    }
}
