#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterAccess {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterDataType {
    Float,
    UInt32,
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterInfo {
    pub rid: u8,
    pub variable: &'static str,
    pub description: &'static str,
    pub access: RegisterAccess,
    pub range_str: &'static str,
    pub data_type: RegisterDataType,
}

macro_rules! reg {
    ($rid:expr, $variable:expr, $description:expr, RW, $range:expr, $typ:ident) => {
        RegisterInfo {
            rid: $rid,
            variable: $variable,
            description: $description,
            access: RegisterAccess::ReadWrite,
            range_str: $range,
            data_type: RegisterDataType::$typ,
        }
    };
    ($rid:expr, $variable:expr, $description:expr, RO, $range:expr, $typ:ident) => {
        RegisterInfo {
            rid: $rid,
            variable: $variable,
            description: $description,
            access: RegisterAccess::ReadOnly,
            range_str: $range,
            data_type: RegisterDataType::$typ,
        }
    };
}

pub static REGISTER_TABLE: &[RegisterInfo] = &[
    reg!(
        0,
        "UV_Value",
        "Under-voltage protection value",
        RW,
        "(10.0, 3.4E38]",
        Float
    ),
    reg!(
        1,
        "KT_Value",
        "Torque coefficient",
        RW,
        "[0.0, 3.4E38]",
        Float
    ),
    reg!(
        2,
        "OT_Value",
        "Over-temperature protection value",
        RW,
        "[80.0, 200)",
        Float
    ),
    reg!(
        3,
        "OC_Value",
        "Over-current protection value",
        RW,
        "(0.0, 1.0)",
        Float
    ),
    reg!(4, "ACC", "Acceleration", RW, "(0.0, 3.4E38)", Float),
    reg!(5, "DEC", "Deceleration", RW, "[-3.4E38, 0.0)", Float),
    reg!(6, "MAX_SPD", "Maximum speed", RW, "(0.0, 3.4E38]", Float),
    reg!(7, "MST_ID", "Feedback ID", RW, "[0, 0x7FF]", UInt32),
    reg!(8, "ESC_ID", "Receive ID", RW, "[0, 0x7FF]", UInt32),
    reg!(
        9,
        "TIMEOUT",
        "Timeout alarm time",
        RW,
        "[0, 2^32-1]",
        UInt32
    ),
    reg!(10, "CTRL_MODE", "Control mode", RW, "[1, 4]", UInt32),
    reg!(
        11,
        "Damp",
        "Motor viscous damping coefficient",
        RO,
        "/",
        Float
    ),
    reg!(12, "Inertia", "Motor moment of inertia", RO, "/", Float),
    reg!(13, "hw_ver", "Reserved", RO, "/", UInt32),
    reg!(14, "sw_ver", "Software version number", RO, "/", UInt32),
    reg!(15, "SN", "Reserved", RO, "/", UInt32),
    reg!(16, "NPP", "Motor pole pairs", RO, "/", UInt32),
    reg!(17, "Rs", "Motor phase resistance", RO, "/", Float),
    reg!(18, "Ls", "Motor phase inductance", RO, "/", Float),
    reg!(19, "Flux", "Motor flux linkage value", RO, "/", Float),
    reg!(20, "Gr", "Gear reduction ratio", RO, "/", Float),
    reg!(
        21,
        "PMAX",
        "Position mapping range",
        RW,
        "(0.0, 3.4E38]",
        Float
    ),
    reg!(
        22,
        "VMAX",
        "Speed mapping range",
        RW,
        "(0.0, 3.4E38]",
        Float
    ),
    reg!(
        23,
        "TMAX",
        "Torque mapping range",
        RW,
        "(0.0, 3.4E38]",
        Float
    ),
    reg!(
        24,
        "I_BW",
        "Current loop control bandwidth",
        RW,
        "[100.0, 10000.0]",
        Float
    ),
    reg!(25, "KP_ASR", "Speed loop Kp", RW, "[0.0, 3.4E38]", Float),
    reg!(26, "KI_ASR", "Speed loop Ki", RW, "[0.0, 3.4E38]", Float),
    reg!(27, "KP_APR", "Position loop Kp", RW, "[0.0, 3.4E38]", Float),
    reg!(28, "KI_APR", "Position loop Ki", RW, "[0.0, 3.4E38]", Float),
    reg!(
        29,
        "OV_Value",
        "Overvoltage protection value",
        RW,
        "TBD",
        Float
    ),
    reg!(
        30,
        "GREF",
        "Gear torque efficiency",
        RW,
        "(0.0, 1.0]",
        Float
    ),
    reg!(
        31,
        "Deta",
        "Speed loop damping coefficient",
        RW,
        "[1.0, 30.0]",
        Float
    ),
    reg!(
        32,
        "V_BW",
        "Speed loop filter bandwidth",
        RW,
        "(0.0, 500.0)",
        Float
    ),
    reg!(
        33,
        "IQ_c1",
        "Current loop enhancement coefficient",
        RW,
        "[100.0, 10000.0]",
        Float
    ),
    reg!(
        34,
        "VL_c1",
        "Speed loop enhancement coefficient",
        RW,
        "(0.0, 10000.0]",
        Float
    ),
    reg!(35, "can_br", "CAN baud rate code", RW, "[0, 4]", UInt32),
    reg!(36, "sub_ver", "Sub-version number", RO, "/", UInt32),
    reg!(50, "u_off", "U-phase offset", RO, "", Float),
    reg!(51, "v_off", "V-phase offset", RO, "", Float),
    reg!(52, "k1", "Compensation factor 1", RO, "", Float),
    reg!(53, "k2", "Compensation factor 2", RO, "", Float),
    reg!(54, "m_off", "Angle offset", RO, "", Float),
    reg!(55, "dir", "Direction", RO, "", Float),
    reg!(56, "m_off", "Motor side angle offset", RO, "", Float),
    reg!(59, "Imax", "Driver board maximum current", RO, "", Float),
    reg!(60, "VBus", "Power supply voltage", RO, "", Float),
    reg!(61, "Tpcb", "Driver board temperature", RO, "", Float),
    reg!(62, "Tmtr", "Motor temperature", RO, "", Float),
    reg!(63, "Iu_off", "U-phase current offset", RO, "", Float),
    reg!(64, "Iv_off", "V-phase current offset", RO, "", Float),
    reg!(65, "Iw_off", "W-phase current offset", RO, "", Float),
    reg!(80, "p_m", "Motor position", RO, "", Float),
    reg!(81, "xout", "Output shaft position", RO, "", Float),
];

pub fn register_info(rid: u8) -> Option<&'static RegisterInfo> {
    REGISTER_TABLE.iter().find(|info| info.rid == rid)
}
