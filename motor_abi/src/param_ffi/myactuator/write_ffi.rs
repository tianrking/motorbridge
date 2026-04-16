use crate::vendor_params::myactuator;

define_param_write_ffis_5!(
    myactuator,
    motor_handle_myactuator_write_param_i8,
    motor_handle_myactuator_write_param_u8,
    motor_handle_myactuator_write_param_u16,
    motor_handle_myactuator_write_param_u32,
    motor_handle_myactuator_write_param_f32
);
