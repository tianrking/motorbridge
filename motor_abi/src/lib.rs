use motor_vendor_damiao::{ControlMode, DamiaoController, DamiaoMotor};
use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::Arc;
use std::time::Duration;

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::new("ok").expect("static cstring"));
}

fn set_last_error(msg: impl AsRef<str>) {
    let clean = msg.as_ref().replace('\0', " ");
    let cstr =
        CString::new(clean).unwrap_or_else(|_| CString::new("error").expect("fallback cstring"));
    LAST_ERROR.with(|slot| *slot.borrow_mut() = cstr);
}

fn ok_ptr() -> *const c_char {
    LAST_ERROR.with(|slot| slot.borrow().as_ptr())
}

fn to_mode(mode: u32) -> Result<ControlMode, &'static str> {
    match mode {
        1 => Ok(ControlMode::Mit),
        2 => Ok(ControlMode::PosVel),
        3 => Ok(ControlMode::Vel),
        4 => Ok(ControlMode::ForcePos),
        _ => Err("mode must be 1(MIT) / 2(POS_VEL) / 3(VEL) / 4(FORCE_POS)"),
    }
}

#[repr(C)]
pub struct MotorController {
    inner: DamiaoController,
}

#[repr(C)]
pub struct MotorHandle {
    inner: Arc<DamiaoMotor>,
}

#[repr(C)]
pub struct MotorState {
    pub has_value: i32,
    pub can_id: u8,
    pub arbitration_id: u16,
    pub status_code: u8,
    pub pos: f32,
    pub vel: f32,
    pub torq: f32,
    pub t_mos: f32,
    pub t_rotor: f32,
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_last_error_message() -> *const c_char {
    ok_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_new_socketcan(channel: *const c_char) -> *mut MotorController {
    if channel.is_null() {
        set_last_error("channel is null");
        return ptr::null_mut();
    }

    let channel = unsafe { CStr::from_ptr(channel) };
    let channel = match channel.to_str() {
        Ok(v) => v,
        Err(_) => {
            set_last_error("channel must be valid UTF-8");
            return ptr::null_mut();
        }
    };

    match DamiaoController::new_socketcan(channel) {
        Ok(controller) => Box::into_raw(Box::new(MotorController { inner: controller })),
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_free(controller: *mut MotorController) {
    if controller.is_null() {
        return;
    }
    let boxed = unsafe { Box::from_raw(controller) };
    let _ = boxed.inner.shutdown();
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_poll_feedback_once(controller: *mut MotorController) -> i32 {
    if controller.is_null() {
        set_last_error("controller is null");
        return -1;
    }
    let controller = unsafe { &mut *controller };
    match controller.inner.poll_feedback_once() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_enable_all(controller: *mut MotorController) -> i32 {
    if controller.is_null() {
        set_last_error("controller is null");
        return -1;
    }
    let controller = unsafe { &mut *controller };
    match controller.inner.enable_all() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_disable_all(controller: *mut MotorController) -> i32 {
    if controller.is_null() {
        set_last_error("controller is null");
        return -1;
    }
    let controller = unsafe { &mut *controller };
    match controller.inner.disable_all() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_shutdown(controller: *mut MotorController) -> i32 {
    if controller.is_null() {
        set_last_error("controller is null");
        return -1;
    }
    let controller = unsafe { &mut *controller };
    match controller.inner.shutdown() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_add_damiao_motor(
    controller: *mut MotorController,
    motor_id: u16,
    feedback_id: u16,
    model: *const c_char,
) -> *mut MotorHandle {
    if controller.is_null() || model.is_null() {
        set_last_error("controller or model is null");
        return ptr::null_mut();
    }

    let controller = unsafe { &mut *controller };
    let model = unsafe { CStr::from_ptr(model) };
    let model = match model.to_str() {
        Ok(v) => v,
        Err(_) => {
            set_last_error("model must be valid UTF-8");
            return ptr::null_mut();
        }
    };

    match controller.inner.add_motor(motor_id, feedback_id, model) {
        Ok(motor) => Box::into_raw(Box::new(MotorHandle { inner: motor })),
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_free(motor: *mut MotorHandle) {
    if motor.is_null() {
        return;
    }
    let _ = unsafe { Box::from_raw(motor) };
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_enable(motor: *mut MotorHandle) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.enable() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_disable(motor: *mut MotorHandle) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.disable() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_clear_error(motor: *mut MotorHandle) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.clear_error() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_set_zero_position(motor: *mut MotorHandle) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.set_zero_position() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_ensure_mode(
    motor: *mut MotorHandle,
    mode: u32,
    timeout_ms: u32,
) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let mode = match to_mode(mode) {
        Ok(v) => v,
        Err(msg) => {
            set_last_error(msg);
            return -1;
        }
    };
    let motor = unsafe { &mut *motor };
    match motor
        .inner
        .ensure_control_mode(mode, Duration::from_millis(timeout_ms as u64))
    {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_send_mit(
    motor: *mut MotorHandle,
    target_position: f32,
    target_velocity: f32,
    stiffness: f32,
    damping: f32,
    feedforward_torque: f32,
) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.send_cmd_mit(
        target_position,
        target_velocity,
        stiffness,
        damping,
        feedforward_torque,
    ) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_send_pos_vel(
    motor: *mut MotorHandle,
    target_position: f32,
    velocity_limit: f32,
) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.send_cmd_pos_vel(target_position, velocity_limit) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_send_vel(motor: *mut MotorHandle, target_velocity: f32) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.send_cmd_vel(target_velocity) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_send_force_pos(
    motor: *mut MotorHandle,
    target_position: f32,
    velocity_limit: f32,
    torque_limit_ratio: f32,
) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor
        .inner
        .send_cmd_force_pos(target_position, velocity_limit, torque_limit_ratio)
    {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_store_parameters(motor: *mut MotorHandle) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.store_parameters() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_request_feedback(motor: *mut MotorHandle) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.request_motor_feedback() {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_set_can_timeout_ms(
    motor: *mut MotorHandle,
    timeout_ms: u32,
) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let reg_value = timeout_ms.saturating_mul(20);
    let motor = unsafe { &mut *motor };
    match motor.inner.write_register_u32(9, reg_value) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_write_register_f32(
    motor: *mut MotorHandle,
    rid: u8,
    value: f32,
) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.write_register_f32(rid, value) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_write_register_u32(
    motor: *mut MotorHandle,
    rid: u8,
    value: u32,
) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    match motor.inner.write_register_u32(rid, value) {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_get_register_f32(
    motor: *mut MotorHandle,
    rid: u8,
    timeout_ms: u32,
    out_value: *mut f32,
) -> i32 {
    if motor.is_null() || out_value.is_null() {
        set_last_error("motor or out_value is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    let out = unsafe { &mut *out_value };
    match motor
        .inner
        .get_register_f32(rid, Duration::from_millis(timeout_ms as u64))
    {
        Ok(v) => {
            *out = v;
            0
        }
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_get_register_u32(
    motor: *mut MotorHandle,
    rid: u8,
    timeout_ms: u32,
    out_value: *mut u32,
) -> i32 {
    if motor.is_null() || out_value.is_null() {
        set_last_error("motor or out_value is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    let out = unsafe { &mut *out_value };
    match motor
        .inner
        .get_register_u32(rid, Duration::from_millis(timeout_ms as u64))
    {
        Ok(v) => {
            *out = v;
            0
        }
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_get_state(
    motor: *mut MotorHandle,
    out_state: *mut MotorState,
) -> i32 {
    if motor.is_null() || out_state.is_null() {
        set_last_error("motor or out_state is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    let out = unsafe { &mut *out_state };
    if let Some(state) = motor.inner.latest_state() {
        *out = MotorState {
            has_value: 1,
            can_id: state.can_id,
            arbitration_id: state.arbitration_id,
            status_code: state.status_code,
            pos: state.pos,
            vel: state.vel,
            torq: state.torq,
            t_mos: state.t_mos,
            t_rotor: state.t_rotor,
        };
    } else {
        *out = MotorState {
            has_value: 0,
            can_id: 0,
            arbitration_id: 0,
            status_code: 0,
            pos: 0.0,
            vel: 0.0,
            torq: 0.0,
            t_mos: 0.0,
            t_rotor: 0.0,
        };
    }
    0
}
