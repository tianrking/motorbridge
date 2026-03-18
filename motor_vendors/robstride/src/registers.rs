#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterDataType {
    Int8,
    UInt8,
    UInt16,
    UInt32,
    Float32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ParameterId {
    MechanicalOffset = 0x2005,
    MeasuredPosition = 0x3016,
    MeasuredVelocity = 0x3017,
    MeasuredTorque = 0x302C,
    Mode = 0x7005,
    IqTarget = 0x7006,
    VelocityTarget = 0x700A,
    TorqueLimit = 0x700B,
    CurrentKp = 0x7010,
    CurrentKi = 0x7011,
    CurrentFilterGain = 0x7014,
    PositionTarget = 0x7016,
    VelocityLimit = 0x7017,
    CurrentLimit = 0x7018,
    MechanicalPosition = 0x7019,
    IqFiltered = 0x701A,
    MechanicalVelocity = 0x701B,
    Vbus = 0x701C,
    PositionKp = 0x701E,
    VelocityKp = 0x701F,
    VelocityKi = 0x7020,
    VelocityFilterGain = 0x7021,
    VelocityAccelerationTarget = 0x7022,
    PpVelocityMax = 0x7024,
    PpAccelerationTarget = 0x7025,
    EpscanTime = 0x7026,
    CanTimeout = 0x7028,
    ZeroState = 0x7029,
}

#[derive(Debug, Clone, Copy)]
pub struct ParameterInfo {
    pub id: u16,
    pub name: &'static str,
    pub data_type: ParameterDataType,
}

macro_rules! param {
    ($id:expr, $name:expr, $ty:ident) => {
        ParameterInfo {
            id: $id,
            name: $name,
            data_type: ParameterDataType::$ty,
        }
    };
}

pub static PARAMETER_TABLE: &[ParameterInfo] = &[
    param!(0x2005, "mechOffset", Float32),
    param!(0x3016, "mechPos_fdb", Float32),
    param!(0x3017, "mechVel_fdb", Float32),
    param!(0x302C, "torque_fdb", Float32),
    param!(0x7005, "run_mode", Int8),
    param!(0x7006, "iq_ref", Float32),
    param!(0x700A, "spd_ref", Float32),
    param!(0x700B, "limit_torque", Float32),
    param!(0x7010, "cur_kp", Float32),
    param!(0x7011, "cur_ki", Float32),
    param!(0x7014, "cur_filter_gain", Float32),
    param!(0x7016, "loc_ref", Float32),
    param!(0x7017, "limit_spd", Float32),
    param!(0x7018, "limit_cur", Float32),
    param!(0x7019, "mechPos", Float32),
    param!(0x701A, "iqf", Float32),
    param!(0x701B, "mechVel", Float32),
    param!(0x701C, "VBUS", Float32),
    param!(0x701E, "loc_kp", Float32),
    param!(0x701F, "spd_kp", Float32),
    param!(0x7020, "spd_ki", Float32),
    param!(0x7021, "spd_filter_gain", Float32),
    param!(0x7022, "acc_rad", Float32),
    param!(0x7024, "vel_max", Float32),
    param!(0x7025, "acc_set", Float32),
    param!(0x7026, "EPScan_time", UInt16),
    param!(0x7028, "canTimeout", UInt32),
    param!(0x7029, "zero_sta", UInt8),
];

pub fn parameter_info(id: u16) -> Option<&'static ParameterInfo> {
    PARAMETER_TABLE.iter().find(|info| info.id == id)
}
