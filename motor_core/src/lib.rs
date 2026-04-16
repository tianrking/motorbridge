pub mod bus;
pub mod controller;
pub mod device;
pub mod dm_serial;
pub mod error;
pub mod model;
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub mod pcan;
pub mod socketcan;
#[cfg(target_os = "linux")]
pub mod socketcanfd;
pub mod test_support;
pub mod vendor_controller;

pub use bus::{CanBus, CanFrame, open_socketcan, open_socketcanfd};
pub use controller::CoreController;
pub use device::MotorDevice;
pub use error::{MotorError, Result};
pub use model::{ModelCatalog, MotorModelSpec, PvTLimits, StaticModelCatalog};
pub use vendor_controller::VendorController;
