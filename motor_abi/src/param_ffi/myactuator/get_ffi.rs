use crate::vendor_params::myactuator;

define_param_get_ffis_5!(
    myactuator,
    motor_handle_myactuator_get_param_i8,
    motor_handle_myactuator_get_param_u8,
    motor_handle_myactuator_get_param_u16,
    motor_handle_myactuator_get_param_u32,
    motor_handle_myactuator_get_param_f32
);
