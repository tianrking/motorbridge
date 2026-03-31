use super::*;

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
pub extern "C" fn motor_controller_add_hexfellow_motor(
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
    match ensure_hexfellow_controller(controller).and_then(|ctrl| {
        ctrl.add_motor(motor_id, feedback_id, &model)
            .map_err(|e| e.to_string())
    }) {
        Ok(motor) => Box::into_raw(Box::new(MotorHandle {
            inner: MotorHandleInner::Hexfellow(motor),
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
pub extern "C" fn motor_controller_add_hightorque_motor(
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
    match ensure_hightorque_controller(controller).and_then(|ctrl| {
        ctrl.add_motor(motor_id, feedback_id, &model)
            .map_err(|e| e.to_string())
    }) {
        Ok(motor) => Box::into_raw(Box::new(MotorHandle {
            inner: MotorHandleInner::Hightorque(motor),
        })),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}
