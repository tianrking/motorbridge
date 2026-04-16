use crate::error::Result;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use crate::pcan::PcanBus;
#[cfg(target_os = "linux")]
use crate::socketcan::SocketCanBus;
#[cfg(target_os = "linux")]
use crate::socketcanfd::SocketCanFdBus;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct CanFrame {
    pub arbitration_id: u32,
    pub data: [u8; 8],
    pub dlc: u8,
    pub is_extended: bool,
    pub is_rx: bool,
}

pub trait CanBus: Send + Sync {
    fn send(&self, frame: CanFrame) -> Result<()>;
    fn recv(&self, timeout: Duration) -> Result<Option<CanFrame>>;
    fn shutdown(&self) -> Result<()>;
}

pub fn open_socketcan(channel: &str) -> Result<Arc<dyn CanBus>> {
    #[cfg(target_os = "linux")]
    {
        let bus: Arc<dyn CanBus> = Arc::new(SocketCanBus::open(channel)?);
        return Ok(bus);
    }
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        let bus: Arc<dyn CanBus> = Arc::new(PcanBus::open(channel)?);
        return Ok(bus);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = channel;
        Err(crate::error::MotorError::InvalidArgument(
            "No CAN backend for current platform".to_string(),
        ))
    }
}

pub fn open_socketcanfd(channel: &str) -> Result<Arc<dyn CanBus>> {
    #[cfg(target_os = "linux")]
    {
        let bus: Arc<dyn CanBus> = Arc::new(SocketCanFdBus::open(channel)?);
        return Ok(bus);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = channel;
        Err(crate::error::MotorError::InvalidArgument(
            "socketcanfd transport is only available on Linux".to_string(),
        ))
    }
}
