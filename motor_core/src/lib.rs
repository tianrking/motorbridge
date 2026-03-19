pub mod bus;
pub mod controller;
pub mod device;
pub mod error;
pub mod model;
#[cfg(target_os = "windows")]
pub mod pcan;
pub mod socketcan;

pub use bus::{CanBus, CanFrame};
pub use controller::CoreController;
pub use device::MotorDevice;
pub use error::{MotorError, Result};
pub use model::{ModelCatalog, MotorModelSpec, PvTLimits, StaticModelCatalog};
