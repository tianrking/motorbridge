use crate::model::{ActiveCommand, ControllerHandle, MotorHandle, Target};

mod connect;
mod runtime;

pub(crate) struct SessionCtx {
    pub(crate) target: Target,
    pub(crate) controller: Option<ControllerHandle>,
    pub(crate) motor: Option<MotorHandle>,
    pub(crate) active: Option<ActiveCommand>,
}

pub(crate) fn myactuator_feedback_default(motor_id: u16) -> u16 {
    0x240u16.saturating_add(motor_id)
}

impl SessionCtx {
    pub(crate) fn model_is_auto(model: &str) -> bool {
        let m = model.trim().to_ascii_lowercase();
        m.is_empty() || m == "auto" || m == "all" || m == "*"
    }

    pub(crate) fn new(target: Target) -> Self {
        Self {
            target,
            controller: None,
            motor: None,
            active: None,
        }
    }
}
