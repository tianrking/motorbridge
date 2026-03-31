use crate::vendor_params::damiao;
use crate::MotorHandle;
use super::super::common::ffi_run;

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_damiao_write_param_f32(
    motor: *mut MotorHandle,
    param_id: u16,
    value: f32,
) -> i32 {
    ffi_run(motor, |m| damiao::write_param_f32(m, param_id, value))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_damiao_write_param_u32(
    motor: *mut MotorHandle,
    param_id: u16,
    value: u32,
) -> i32 {
    ffi_run(motor, |m| damiao::write_param_u32(m, param_id, value))
}
