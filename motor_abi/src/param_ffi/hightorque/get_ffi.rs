use crate::vendor_params::hightorque;
use crate::MotorHandle;
use super::super::common::ffi_get;

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_hightorque_get_param_i8(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut i8,
) -> i32 {
    ffi_get(motor, out_value, |m| hightorque::get_i8(m, param_id, timeout_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_hightorque_get_param_u8(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut u8,
) -> i32 {
    ffi_get(motor, out_value, |m| hightorque::get_u8(m, param_id, timeout_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_hightorque_get_param_u16(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut u16,
) -> i32 {
    ffi_get(motor, out_value, |m| hightorque::get_u16(m, param_id, timeout_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_hightorque_get_param_u32(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut u32,
) -> i32 {
    ffi_get(motor, out_value, |m| hightorque::get_u32(m, param_id, timeout_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_hightorque_get_param_f32(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut f32,
) -> i32 {
    ffi_get(motor, out_value, |m| hightorque::get_f32(m, param_id, timeout_ms))
}
