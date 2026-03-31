use crate::{set_last_error, MotorHandle};

pub(crate) fn ffi_get<T, F>(motor: *mut MotorHandle, out_value: *mut T, f: F) -> i32
where
    F: FnOnce(&mut MotorHandle) -> Result<T, String>,
{
    if motor.is_null() || out_value.is_null() {
        set_last_error("motor or out_value is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    let out = unsafe { &mut *out_value };
    match f(motor) {
        Ok(v) => {
            *out = v;
            0
        }
        Err(e) => {
            set_last_error(e);
            -1
        }
    }
}

pub(crate) fn ffi_run<F>(motor: *mut MotorHandle, f: F) -> i32
where
    F: FnOnce(&mut MotorHandle) -> Result<(), String>,
{
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match f(motor) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
            -1
        }
    }
}
