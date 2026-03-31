use crate::vendor_params::robstride;
use crate::MotorHandle;
use super::super::common::ffi_run;

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_param_write_i8(
    motor: *mut MotorHandle,
    param_id: u16,
    value: i8,
) -> i32 {
    ffi_run(motor, |m| robstride::write_i8(m, param_id, value))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_param_write_u8(
    motor: *mut MotorHandle,
    param_id: u16,
    value: u8,
) -> i32 {
    ffi_run(motor, |m| robstride::write_u8(m, param_id, value))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_param_write_u16(
    motor: *mut MotorHandle,
    param_id: u16,
    value: u16,
) -> i32 {
    ffi_run(motor, |m| robstride::write_u16(m, param_id, value))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_param_write_u32(
    motor: *mut MotorHandle,
    param_id: u16,
    value: u32,
) -> i32 {
    ffi_run(motor, |m| robstride::write_u32(m, param_id, value))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_param_write_f32(
    motor: *mut MotorHandle,
    param_id: u16,
    value: f32,
) -> i32 {
    ffi_run(motor, |m| robstride::write_f32(m, param_id, value))
}
