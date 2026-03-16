pub mod controller;
pub mod motor;
pub mod protocol;
pub mod registers;

pub use controller::DamiaoController;
pub use motor::{
    match_models_by_limits, model_limits, suggest_models_by_limits, ControlMode, DamiaoMotor,
    MotorFeedbackState, RegisterValue,
};
pub use protocol::{
    decode_sensor_feedback, encode_force_pos_cmd, encode_mit_cmd, encode_pos_vel_cmd,
    encode_vel_cmd,
};
pub use registers::{
    register_info, RegisterAccess, RegisterDataType, RegisterInfo, REGISTER_TABLE,
};
