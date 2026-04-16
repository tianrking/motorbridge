use crate::vendor_params::hightorque;

define_param_write_ffis_5!(
    hightorque,
    motor_handle_hightorque_write_param_i8,
    motor_handle_hightorque_write_param_u8,
    motor_handle_hightorque_write_param_u16,
    motor_handle_hightorque_write_param_u32,
    motor_handle_hightorque_write_param_f32
);
