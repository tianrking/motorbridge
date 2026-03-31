use crate::vendor_params::damiao;
use crate::MotorHandle;
use super::super::common::ffi_get;

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_damiao_get_param_f32(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut f32,
) -> i32 {
    ffi_get(motor, out_value, |m| damiao::get_param_f32(m, param_id, timeout_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_damiao_get_param_u32(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut u32,
) -> i32 {
    ffi_get(motor, out_value, |m| damiao::get_param_u32(m, param_id, timeout_ms))
}
