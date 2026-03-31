use super::*;

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
pub extern "C" fn motor_controller_new_socketcanfd(channel: *const c_char) -> *mut MotorController {
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
pub extern "C" fn motor_controller_new_dm_serial(
    serial_port: *const c_char,
    baud: u32,
) -> *mut MotorController {
    let serial_port = match parse_cstr(serial_port, "serial_port") {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return ptr::null_mut();
        }
    };
    let controller = match DamiaoController::new_dm_serial(&serial_port, baud) {
        Ok(c) => c,
        Err(e) => {
            set_last_error(e.to_string());
            return ptr::null_mut();
        }
    };
    Box::into_raw(Box::new(MotorController {
        inner: ControllerInner::Damiao(controller),
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
        ControllerInner::Hexfellow(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string()),
        ControllerInner::MyActuator(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string()),
        ControllerInner::Robstride(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string()),
        ControllerInner::Hightorque(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string()),
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
        ControllerInner::Hexfellow(ctrl) => ctrl.enable_all().map_err(|e| e.to_string()),
        ControllerInner::MyActuator(ctrl) => ctrl.enable_all().map_err(|e| e.to_string()),
        ControllerInner::Robstride(ctrl) => ctrl.enable_all().map_err(|e| e.to_string()),
        ControllerInner::Hightorque(ctrl) => ctrl.enable_all().map_err(|e| e.to_string()),
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
        ControllerInner::Hexfellow(ctrl) => ctrl.disable_all().map_err(|e| e.to_string()),
        ControllerInner::MyActuator(ctrl) => ctrl.disable_all().map_err(|e| e.to_string()),
        ControllerInner::Robstride(ctrl) => ctrl.disable_all().map_err(|e| e.to_string()),
        ControllerInner::Hightorque(ctrl) => ctrl.disable_all().map_err(|e| e.to_string()),
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
        ControllerInner::Hexfellow(ctrl) => ctrl.shutdown().map_err(|e| e.to_string()),
        ControllerInner::MyActuator(ctrl) => ctrl.shutdown().map_err(|e| e.to_string()),
        ControllerInner::Robstride(ctrl) => ctrl.shutdown().map_err(|e| e.to_string()),
        ControllerInner::Hightorque(ctrl) => ctrl.shutdown().map_err(|e| e.to_string()),
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
        ControllerInner::Hexfellow(ctrl) => ctrl.close_bus().map_err(|e| e.to_string()),
        ControllerInner::MyActuator(ctrl) => ctrl.close_bus().map_err(|e| e.to_string()),
        ControllerInner::Robstride(ctrl) => ctrl.close_bus().map_err(|e| e.to_string()),
        ControllerInner::Hightorque(ctrl) => ctrl.close_bus().map_err(|e| e.to_string()),
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
