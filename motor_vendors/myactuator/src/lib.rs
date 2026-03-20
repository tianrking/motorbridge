pub mod controller;
pub mod motor;
pub mod protocol;
pub mod registers;

pub use controller::MyActuatorController;
pub use motor::{ControlMode, MyActuatorFeedbackState, MyActuatorMotor};
pub use protocol::Command;
