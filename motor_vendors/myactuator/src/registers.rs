#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AccelerationType {
    PositionPlanningAcceleration = 0x00,
    PositionPlanningDeceleration = 0x01,
    VelocityPlanningAcceleration = 0x02,
    VelocityPlanningDeceleration = 0x03,
}
