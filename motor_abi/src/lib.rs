use motor_vendor_damiao::{ControlMode as DamiaoControlMode, DamiaoController, DamiaoMotor};
use motor_vendor_hexfellow::{
    HexfellowController, HexfellowMotor, MitTarget as HexfellowMitTarget,
    PosVelTarget as HexfellowPosVelTarget,
};
use motor_vendor_hightorque::{HightorqueController, HightorqueMotor};
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
    Hexfellow(HexfellowController),
    MyActuator(MyActuatorController),
    Robstride(RobstrideController),
    Hightorque(HightorqueController),
}

enum MotorHandleInner {
    Damiao(Arc<DamiaoMotor>),
    Hexfellow(Arc<HexfellowMotor>),
    MyActuator(Arc<MyActuatorMotor>),
    Robstride(Arc<RobstrideMotor>),
    Hightorque(Arc<HightorqueMotor>),
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
        ControllerInner::Hexfellow(_) => {
            Err("controller already bound to Hexfellow; use a separate controller".to_string())
        }
        ControllerInner::MyActuator(_) => {
            Err("controller already bound to MyActuator; use a separate controller".to_string())
        }
        ControllerInner::Robstride(_) => {
            Err("controller already bound to RobStride; use a separate controller".to_string())
        }
        ControllerInner::Hightorque(_) => {
            Err("controller already bound to HighTorque; use a separate controller".to_string())
        }
        ControllerInner::Unbound(_) => Err("controller binding failed".to_string()),
    }
}

fn ensure_hexfellow_controller(
    controller: &mut MotorController,
) -> Result<&mut HexfellowController, String> {
    if let ControllerInner::Unbound(channel) = &controller.inner {
        controller.inner = ControllerInner::Hexfellow(
            HexfellowController::new_socketcanfd(channel).map_err(|e| e.to_string())?,
        );
    }
    match &mut controller.inner {
        ControllerInner::Hexfellow(ctrl) => Ok(ctrl),
        ControllerInner::Damiao(_) => {
            Err("controller already bound to Damiao; use a separate controller".to_string())
        }
        ControllerInner::MyActuator(_) => {
            Err("controller already bound to MyActuator; use a separate controller".to_string())
        }
        ControllerInner::Robstride(_) => {
            Err("controller already bound to RobStride; use a separate controller".to_string())
        }
        ControllerInner::Hightorque(_) => {
            Err("controller already bound to HighTorque; use a separate controller".to_string())
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
        ControllerInner::Hexfellow(_) => {
            Err("controller already bound to Hexfellow; use a separate controller".to_string())
        }
        ControllerInner::Robstride(_) => {
            Err("controller already bound to RobStride; use a separate controller".to_string())
        }
        ControllerInner::Hightorque(_) => {
            Err("controller already bound to HighTorque; use a separate controller".to_string())
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
        ControllerInner::Hexfellow(_) => {
            Err("controller already bound to Hexfellow; use a separate controller".to_string())
        }
        ControllerInner::MyActuator(_) => {
            Err("controller already bound to MyActuator; use a separate controller".to_string())
        }
        ControllerInner::Hightorque(_) => {
            Err("controller already bound to HighTorque; use a separate controller".to_string())
        }
        ControllerInner::Unbound(_) => Err("controller binding failed".to_string()),
    }
}

fn ensure_hightorque_controller(
    controller: &mut MotorController,
) -> Result<&mut HightorqueController, String> {
    if let ControllerInner::Unbound(channel) = &controller.inner {
        controller.inner = ControllerInner::Hightorque(
            HightorqueController::new_socketcan(channel).map_err(|e| e.to_string())?,
        );
    }
    match &mut controller.inner {
        ControllerInner::Hightorque(ctrl) => Ok(ctrl),
        ControllerInner::Damiao(_) => {
            Err("controller already bound to Damiao; use a separate controller".to_string())
        }
        ControllerInner::Hexfellow(_) => {
            Err("controller already bound to Hexfellow; use a separate controller".to_string())
        }
        ControllerInner::MyActuator(_) => {
            Err("controller already bound to MyActuator; use a separate controller".to_string())
        }
        ControllerInner::Robstride(_) => {
            Err("controller already bound to RobStride; use a separate controller".to_string())
        }
        ControllerInner::Unbound(_) => Err("controller binding failed".to_string()),
    }
}


mod controller_add_motor_ffi;
mod controller_lifecycle_ffi;
mod motor_control_ffi;
mod motor_lifecycle_ffi;
mod motor_register_ffi;
mod param_ffi;
mod state_ffi;
mod vendor_params;
