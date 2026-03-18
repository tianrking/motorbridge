use crate::bus::CanFrame;
use crate::error::Result;

pub trait MotorDevice: Send + Sync {
    fn vendor(&self) -> &'static str;
    fn model(&self) -> &str;
    fn motor_id(&self) -> u16;
    fn feedback_id(&self) -> u16;

    fn enable(&self) -> Result<()>;
    fn disable(&self) -> Result<()>;

    fn accepts_frame(&self, frame: &CanFrame) -> bool;
    fn process_feedback_frame(&self, frame: CanFrame) -> Result<()>;
}
