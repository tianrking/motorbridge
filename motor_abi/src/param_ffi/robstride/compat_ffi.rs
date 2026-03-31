use crate::MotorHandle;

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_get_param_i8(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut i8,
) -> i32 {
    super::get_ffi::motor_handle_robstride_param_get_i8(motor, param_id, timeout_ms, out_value)
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_get_param_u8(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut u8,
) -> i32 {
    super::get_ffi::motor_handle_robstride_param_get_u8(motor, param_id, timeout_ms, out_value)
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_get_param_u16(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut u16,
) -> i32 {
    super::get_ffi::motor_handle_robstride_param_get_u16(motor, param_id, timeout_ms, out_value)
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_get_param_u32(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut u32,
) -> i32 {
    super::get_ffi::motor_handle_robstride_param_get_u32(motor, param_id, timeout_ms, out_value)
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_get_param_f32(
    motor: *mut MotorHandle,
    param_id: u16,
    timeout_ms: u32,
    out_value: *mut f32,
) -> i32 {
    super::get_ffi::motor_handle_robstride_param_get_f32(motor, param_id, timeout_ms, out_value)
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_write_param_i8(
    motor: *mut MotorHandle,
    param_id: u16,
    value: i8,
) -> i32 {
    super::write_ffi::motor_handle_robstride_param_write_i8(motor, param_id, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_write_param_u8(
    motor: *mut MotorHandle,
    param_id: u16,
    value: u8,
) -> i32 {
    super::write_ffi::motor_handle_robstride_param_write_u8(motor, param_id, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_write_param_u16(
    motor: *mut MotorHandle,
    param_id: u16,
    value: u16,
) -> i32 {
    super::write_ffi::motor_handle_robstride_param_write_u16(motor, param_id, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_write_param_u32(
    motor: *mut MotorHandle,
    param_id: u16,
    value: u32,
) -> i32 {
    super::write_ffi::motor_handle_robstride_param_write_u32(motor, param_id, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_write_param_f32(
    motor: *mut MotorHandle,
    param_id: u16,
    value: f32,
) -> i32 {
    super::write_ffi::motor_handle_robstride_param_write_f32(motor, param_id, value)
}
