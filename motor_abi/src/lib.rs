use motor_vendor_damiao::{ControlMode as DamiaoControlMode, DamiaoController, DamiaoMotor};
use motor_vendor_myactuator::{
    ControlMode as MyActuatorControlMode, MyActuatorController, MyActuatorMotor,
};
use motor_vendor_robstride::{
    ControlMode as RobstrideControlMode, ParameterValue, RobstrideController, RobstrideMotor,
};
use std::cell::RefCell;
use std::f32::consts::PI;
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

fn to_damiao_mode(mode: u32) -> Result<DamiaoControlMode, &'static str> {
    match mode {
        1 => Ok(DamiaoControlMode::Mit),
        2 => Ok(DamiaoControlMode::PosVel),
        3 => Ok(DamiaoControlMode::Vel),
        4 => Ok(DamiaoControlMode::ForcePos),
        _ => Err("Damiao mode must be 1(MIT) / 2(POS_VEL) / 3(VEL) / 4(FORCE_POS)"),
    }
}

fn to_robstride_mode(mode: u32) -> Result<RobstrideControlMode, &'static str> {
    match mode {
        1 => Ok(RobstrideControlMode::Mit),
        2 => Ok(RobstrideControlMode::Position),
        3 => Ok(RobstrideControlMode::Velocity),
        _ => Err("RobStride mode must be 1(MIT) / 2(POSITION) / 3(VELOCITY)"),
    }
}

fn to_myactuator_mode(mode: u32) -> Result<MyActuatorControlMode, &'static str> {
    match mode {
        1 => Ok(MyActuatorControlMode::Current),
        2 => Ok(MyActuatorControlMode::Position),
        3 => Ok(MyActuatorControlMode::Velocity),
        _ => Err("MyActuator mode must be 1(CURRENT) / 2(POSITION) / 3(VELOCITY)"),
    }
}

enum ControllerInner {
    Unbound(String),
    Damiao(DamiaoController),
    MyActuator(MyActuatorController),
    Robstride(RobstrideController),
}

enum MotorHandleInner {
    Damiao(Arc<DamiaoMotor>),
    MyActuator(Arc<MyActuatorMotor>),
    Robstride(Arc<RobstrideMotor>),
}

#[repr(C)]
pub struct MotorController {
    inner: ControllerInner,
}

#[repr(C)]
pub struct MotorHandle {
    inner: MotorHandleInner,
}

#[repr(C)]
pub struct MotorState {
    pub has_value: i32,
    pub can_id: u8,
    pub arbitration_id: u32,
    pub status_code: u8,
    pub pos: f32,
    pub vel: f32,
    pub torq: f32,
    pub t_mos: f32,
    pub t_rotor: f32,
}

fn parse_cstr(ptr: *const c_char, name: &str) -> Result<String, String> {
    if ptr.is_null() {
        return Err(format!("{name} is null"));
    }
    let s = unsafe { CStr::from_ptr(ptr) };
    s.to_str()
        .map(|v| v.to_string())
        .map_err(|_| format!("{name} must be valid UTF-8"))
}

fn ensure_damiao_controller(
    controller: &mut MotorController,
) -> Result<&mut DamiaoController, String> {
    if let ControllerInner::Unbound(channel) = &controller.inner {
        controller.inner = ControllerInner::Damiao(
            DamiaoController::new_socketcan(channel).map_err(|e| e.to_string())?,
        );
    }
    match &mut controller.inner {
        ControllerInner::Damiao(ctrl) => Ok(ctrl),
        ControllerInner::MyActuator(_) => {
            Err("controller already bound to MyActuator; use a separate controller".to_string())
        }
        ControllerInner::Robstride(_) => {
            Err("controller already bound to RobStride; use a separate controller".to_string())
        }
        ControllerInner::Unbound(_) => Err("controller binding failed".to_string()),
    }
}

fn ensure_myactuator_controller(
    controller: &mut MotorController,
) -> Result<&mut MyActuatorController, String> {
    if let ControllerInner::Unbound(channel) = &controller.inner {
        controller.inner = ControllerInner::MyActuator(
            MyActuatorController::new_socketcan(channel).map_err(|e| e.to_string())?,
        );
    }
    match &mut controller.inner {
        ControllerInner::MyActuator(ctrl) => Ok(ctrl),
        ControllerInner::Damiao(_) => {
            Err("controller already bound to Damiao; use a separate controller".to_string())
        }
        ControllerInner::Robstride(_) => {
            Err("controller already bound to RobStride; use a separate controller".to_string())
        }
        ControllerInner::Unbound(_) => Err("controller binding failed".to_string()),
    }
}

fn ensure_robstride_controller(
    controller: &mut MotorController,
) -> Result<&mut RobstrideController, String> {
    if let ControllerInner::Unbound(channel) = &controller.inner {
        controller.inner = ControllerInner::Robstride(
            RobstrideController::new_socketcan(channel).map_err(|e| e.to_string())?,
        );
    }
    match &mut controller.inner {
        ControllerInner::Robstride(ctrl) => Ok(ctrl),
        ControllerInner::Damiao(_) => {
            Err("controller already bound to Damiao; use a separate controller".to_string())
        }
        ControllerInner::MyActuator(_) => {
            Err("controller already bound to MyActuator; use a separate controller".to_string())
        }
        ControllerInner::Unbound(_) => Err("controller binding failed".to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_last_error_message() -> *const c_char {
    ok_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_new_socketcan(channel: *const c_char) -> *mut MotorController {
    let channel = match parse_cstr(channel, "channel") {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };
    Box::into_raw(Box::new(MotorController {
        inner: ControllerInner::Unbound(channel),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_free(controller: *mut MotorController) {
    if controller.is_null() {
        return;
    }
    let _ = unsafe { Box::from_raw(controller) };
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_poll_feedback_once(controller: *mut MotorController) -> i32 {
    if controller.is_null() {
        set_last_error("controller is null");
        return -1;
    }
    let controller = unsafe { &mut *controller };
    let rc = match &mut controller.inner {
        ControllerInner::Damiao(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string()),
        ControllerInner::MyActuator(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string()),
        ControllerInner::Robstride(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string()),
        ControllerInner::Unbound(_) => Ok(()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &mut controller.inner {
        ControllerInner::Damiao(ctrl) => ctrl.enable_all().map_err(|e| e.to_string()),
        ControllerInner::MyActuator(ctrl) => ctrl.enable_all().map_err(|e| e.to_string()),
        ControllerInner::Robstride(ctrl) => ctrl.enable_all().map_err(|e| e.to_string()),
        ControllerInner::Unbound(_) => Ok(()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &mut controller.inner {
        ControllerInner::Damiao(ctrl) => ctrl.disable_all().map_err(|e| e.to_string()),
        ControllerInner::MyActuator(ctrl) => ctrl.disable_all().map_err(|e| e.to_string()),
        ControllerInner::Robstride(ctrl) => ctrl.disable_all().map_err(|e| e.to_string()),
        ControllerInner::Unbound(_) => Ok(()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &mut controller.inner {
        ControllerInner::Damiao(ctrl) => ctrl.shutdown().map_err(|e| e.to_string()),
        ControllerInner::MyActuator(ctrl) => ctrl.shutdown().map_err(|e| e.to_string()),
        ControllerInner::Robstride(ctrl) => ctrl.shutdown().map_err(|e| e.to_string()),
        ControllerInner::Unbound(_) => Ok(()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_close_bus(controller: *mut MotorController) -> i32 {
    if controller.is_null() {
        set_last_error("controller is null");
        return -1;
    }
    let controller = unsafe { &mut *controller };
    let rc = match &mut controller.inner {
        ControllerInner::Damiao(ctrl) => ctrl.close_bus().map_err(|e| e.to_string()),
        ControllerInner::MyActuator(ctrl) => ctrl.close_bus().map_err(|e| e.to_string()),
        ControllerInner::Robstride(ctrl) => ctrl.close_bus().map_err(|e| e.to_string()),
        ControllerInner::Unbound(_) => Ok(()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    if controller.is_null() {
        set_last_error("controller is null");
        return ptr::null_mut();
    }
    let model = match parse_cstr(model, "model") {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };
    let controller = unsafe { &mut *controller };
    match ensure_damiao_controller(controller).and_then(|ctrl| {
        ctrl.add_motor(motor_id, feedback_id, &model)
            .map_err(|e| e.to_string())
    }) {
        Ok(motor) => Box::into_raw(Box::new(MotorHandle {
            inner: MotorHandleInner::Damiao(motor),
        })),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_add_robstride_motor(
    controller: *mut MotorController,
    motor_id: u16,
    feedback_id: u16,
    model: *const c_char,
) -> *mut MotorHandle {
    if controller.is_null() {
        set_last_error("controller is null");
        return ptr::null_mut();
    }
    let model = match parse_cstr(model, "model") {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };
    let controller = unsafe { &mut *controller };
    match ensure_robstride_controller(controller).and_then(|ctrl| {
        ctrl.add_motor(motor_id, feedback_id, &model)
            .map_err(|e| e.to_string())
    }) {
        Ok(motor) => Box::into_raw(Box::new(MotorHandle {
            inner: MotorHandleInner::Robstride(motor),
        })),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_controller_add_myactuator_motor(
    controller: *mut MotorController,
    motor_id: u16,
    feedback_id: u16,
    model: *const c_char,
) -> *mut MotorHandle {
    if controller.is_null() {
        set_last_error("controller is null");
        return ptr::null_mut();
    }
    let model = match parse_cstr(model, "model") {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };
    let controller = unsafe { &mut *controller };
    match ensure_myactuator_controller(controller).and_then(|ctrl| {
        ctrl.add_motor(motor_id, feedback_id, &model)
            .map_err(|e| e.to_string())
    }) {
        Ok(motor) => Box::into_raw(Box::new(MotorHandle {
            inner: MotorHandleInner::MyActuator(motor),
        })),
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m.enable().map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(m) => m.release_brake().map_err(|e| e.to_string()),
        MotorHandleInner::Robstride(m) => m.enable().map_err(|e| e.to_string()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m.disable().map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(m) => m.shutdown_motor().map_err(|e| e.to_string()),
        MotorHandleInner::Robstride(m) => m.disable().map_err(|e| e.to_string()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m.clear_error().map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(m) => m.stop_motor().map_err(|e| e.to_string()),
        MotorHandleInner::Robstride(_) => {
            Err("clear_error is not supported for RobStride ABI yet".to_string())
        }
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m.set_zero_position().map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(_) => {
            Err("set_zero_position is not supported for MyActuator".to_string())
        }
        MotorHandleInner::Robstride(m) => m.set_zero_position().map_err(|e| e.to_string()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let motor = unsafe { &mut *motor };
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => {
            let mode = match to_damiao_mode(mode) {
                Ok(v) => v,
                Err(e) => {
                    return {
                        set_last_error(e);
                        -1
                    }
                }
            };
            m.ensure_control_mode(mode, Duration::from_millis(timeout_ms as u64))
                .map_err(|e| e.to_string())
        }
        MotorHandleInner::MyActuator(_m) => match to_myactuator_mode(mode) {
            Ok(_mode) => Ok(()),
            Err(e) => {
                set_last_error(e);
                return -1;
            }
        },
        MotorHandleInner::Robstride(m) => {
            let mode = match to_robstride_mode(mode) {
                Ok(v) => v,
                Err(e) => {
                    return {
                        set_last_error(e);
                        -1
                    }
                }
            };
            m.set_mode(mode).map_err(|e| e.to_string())
        }
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m
            .send_cmd_mit(
                target_position,
                target_velocity,
                stiffness,
                damping,
                feedforward_torque,
            )
            .map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(_) => {
            Err("send_mit is not supported for MyActuator; use pos-vel or vel".to_string())
        }
        MotorHandleInner::Robstride(m) => m
            .send_cmd_mit(
                target_position,
                target_velocity,
                stiffness,
                damping,
                feedforward_torque,
            )
            .map_err(|e| e.to_string()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m
            .send_cmd_pos_vel(target_position, velocity_limit)
            .map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(m) => m
            .send_position_absolute_setpoint(
                target_position * (180.0 / PI),
                velocity_limit * (180.0 / PI),
            )
            .map_err(|e| e.to_string()),
        MotorHandleInner::Robstride(_) => {
            Err("send_pos_vel is not supported for RobStride".to_string())
        }
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m.send_cmd_vel(target_velocity).map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(m) => m
            .send_velocity_setpoint(target_velocity * (180.0 / PI))
            .map_err(|e| e.to_string()),
        MotorHandleInner::Robstride(m) => m
            .set_velocity_target(target_velocity)
            .map_err(|e| e.to_string()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m
            .send_cmd_force_pos(target_position, velocity_limit, torque_limit_ratio)
            .map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(_) => {
            Err("send_force_pos is not supported for MyActuator".to_string())
        }
        MotorHandleInner::Robstride(_) => {
            Err("send_force_pos is not supported for RobStride".to_string())
        }
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m.store_parameters().map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(_) => {
            Err("store_parameters is not supported for MyActuator".to_string())
        }
        MotorHandleInner::Robstride(m) => m.save_parameters().map_err(|e| e.to_string()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m.request_motor_feedback().map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(m) => m.request_status().map_err(|e| e.to_string()),
        MotorHandleInner::Robstride(_) => Err("request_feedback is not supported for RobStride; status arrives from operation replies".to_string()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_set_can_timeout_ms(motor: *mut MotorHandle, timeout_ms: u32) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let reg_value = timeout_ms.saturating_mul(20);
    let motor = unsafe { &mut *motor };
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m
            .write_register_u32(9, reg_value)
            .map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(_) => {
            Err("set_can_timeout_ms is not supported for MyActuator".to_string())
        }
        MotorHandleInner::Robstride(m) => m
            .write_parameter(0x7028, ParameterValue::U32(timeout_ms))
            .map_err(|e| e.to_string()),
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m.write_register_f32(rid, value).map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(_) => {
            Err("register write is not available for MyActuator".to_string())
        }
        MotorHandleInner::Robstride(_) => {
            Err("Damiao register write is not available for RobStride".to_string())
        }
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m.write_register_u32(rid, value).map_err(|e| e.to_string()),
        MotorHandleInner::MyActuator(_) => {
            Err("register write is not available for MyActuator".to_string())
        }
        MotorHandleInner::Robstride(_) => {
            Err("Damiao register write is not available for RobStride".to_string())
        }
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m
            .get_register_f32(rid, Duration::from_millis(timeout_ms as u64))
            .map_err(|e| e.to_string())
            .map(|v| *out = v),
        MotorHandleInner::MyActuator(_) => {
            Err("register read is not available for MyActuator".to_string())
        }
        MotorHandleInner::Robstride(_) => {
            Err("Damiao register read is not available for RobStride".to_string())
        }
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
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
    let rc = match &motor.inner {
        MotorHandleInner::Damiao(m) => m
            .get_register_u32(rid, Duration::from_millis(timeout_ms as u64))
            .map_err(|e| e.to_string())
            .map(|v| *out = v),
        MotorHandleInner::MyActuator(_) => {
            Err("register read is not available for MyActuator".to_string())
        }
        MotorHandleInner::Robstride(_) => {
            Err("Damiao register read is not available for RobStride".to_string())
        }
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_ping(
    motor: *mut MotorHandle,
    out_device_id: *mut u8,
    out_responder_id: *mut u8,
) -> i32 {
    if motor.is_null() || out_device_id.is_null() || out_responder_id.is_null() {
        set_last_error("motor or output pointer is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    let rc = match &motor.inner {
        MotorHandleInner::Robstride(m) => m
            .ping(Duration::from_millis(500))
            .map_err(|e| e.to_string()),
        MotorHandleInner::Damiao(_) | MotorHandleInner::MyActuator(_) => {
            Err("robstride_ping requires a RobStride motor".to_string())
        }
    };
    match rc {
        Ok(reply) => {
            unsafe {
                *out_device_id = reply.device_id;
                *out_responder_id = reply.responder_id;
            }
            0
        }
        Err(e) => {
            set_last_error(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn motor_handle_robstride_set_device_id(
    motor: *mut MotorHandle,
    new_device_id: u8,
) -> i32 {
    if motor.is_null() {
        set_last_error("motor is null");
        return -1;
    }
    let motor = unsafe { &mut *motor };
    let rc = match &motor.inner {
        MotorHandleInner::Robstride(m) => m.set_device_id(new_device_id).map_err(|e| e.to_string()),
        MotorHandleInner::Damiao(_) | MotorHandleInner::MyActuator(_) => {
            Err("robstride_set_device_id requires a RobStride motor".to_string())
        }
    };
    match rc {
        Ok(()) => 0,
        Err(e) => {
            set_last_error(e);
            -1
        }
    }
}

macro_rules! robstride_get_param {
    ($name:ident, $variant:ident, $out_ty:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(
            motor: *mut MotorHandle,
            param_id: u16,
            timeout_ms: u32,
            out_value: *mut $out_ty,
        ) -> i32 {
            if motor.is_null() || out_value.is_null() {
                set_last_error("motor or out_value is null");
                return -1;
            }
            let motor = unsafe { &mut *motor };
            let rc = match &motor.inner {
                MotorHandleInner::Robstride(m) => m
                    .get_parameter(param_id, Duration::from_millis(timeout_ms as u64))
                    .map_err(|e| e.to_string()),
                MotorHandleInner::Damiao(_) | MotorHandleInner::MyActuator(_) => {
                    Err("RobStride parameter access requires a RobStride motor".to_string())
                }
            };
            match rc {
                Ok(ParameterValue::$variant(v)) => {
                    unsafe {
                        *out_value = v;
                    }
                    0
                }
                Ok(_) => {
                    set_last_error("RobStride parameter type mismatch");
                    -1
                }
                Err(e) => {
                    set_last_error(e);
                    -1
                }
            }
        }
    };
}

macro_rules! robstride_write_param {
    ($name:ident, $variant:ident, $in_ty:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(motor: *mut MotorHandle, param_id: u16, value: $in_ty) -> i32 {
            if motor.is_null() {
                set_last_error("motor is null");
                return -1;
            }
            let motor = unsafe { &mut *motor };
            let rc = match &motor.inner {
                MotorHandleInner::Robstride(m) => m
                    .write_parameter(param_id, ParameterValue::$variant(value))
                    .map_err(|e| e.to_string()),
                MotorHandleInner::Damiao(_) | MotorHandleInner::MyActuator(_) => {
                    Err("RobStride parameter access requires a RobStride motor".to_string())
                }
            };
            match rc {
                Ok(()) => 0,
                Err(e) => {
                    set_last_error(e);
                    -1
                }
            }
        }
    };
}

robstride_get_param!(motor_handle_robstride_get_param_i8, I8, i8);
robstride_get_param!(motor_handle_robstride_get_param_u8, U8, u8);
robstride_get_param!(motor_handle_robstride_get_param_u16, U16, u16);
robstride_get_param!(motor_handle_robstride_get_param_u32, U32, u32);
robstride_get_param!(motor_handle_robstride_get_param_f32, F32, f32);

robstride_write_param!(motor_handle_robstride_write_param_i8, I8, i8);
robstride_write_param!(motor_handle_robstride_write_param_u8, U8, u8);
robstride_write_param!(motor_handle_robstride_write_param_u16, U16, u16);
robstride_write_param!(motor_handle_robstride_write_param_u32, U32, u32);
robstride_write_param!(motor_handle_robstride_write_param_f32, F32, f32);

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
    match &motor.inner {
        MotorHandleInner::Damiao(m) => {
            if let Some(state) = m.latest_state() {
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
        }
        MotorHandleInner::MyActuator(m) => {
            if let Some(state) = m.latest_state() {
                *out = MotorState {
                    has_value: 1,
                    can_id: m.motor_id as u8,
                    arbitration_id: state.arbitration_id,
                    status_code: state.command,
                    pos: state.shaft_angle_deg * (PI / 180.0),
                    vel: state.speed_dps * (PI / 180.0),
                    torq: state.current_a,
                    t_mos: f32::from(state.temperature_c),
                    t_rotor: 0.0,
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
        }
        MotorHandleInner::Robstride(m) => {
            if let Some(state) = m.latest_state() {
                let mut status = 0u8;
                if state.uncalibrated {
                    status |= 1 << 5;
                }
                if state.stall {
                    status |= 1 << 4;
                }
                if state.magnetic_encoder_fault {
                    status |= 1 << 3;
                }
                if state.overtemperature {
                    status |= 1 << 2;
                }
                if state.overcurrent {
                    status |= 1 << 1;
                }
                if state.undervoltage {
                    status |= 1;
                }
                *out = MotorState {
                    has_value: 1,
                    can_id: state.device_id,
                    arbitration_id: state.arbitration_id,
                    status_code: status,
                    pos: state.position,
                    vel: state.velocity,
                    torq: state.torque,
                    t_mos: state.temperature_c,
                    t_rotor: 0.0,
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
        }
    }
    0
}
