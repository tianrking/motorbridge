from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class RegisterSpec:
    rid: int
    variable: str
    description: str
    data_type: str
    access: str
    range_str: str


# Damiao RW register subset used for configuration/tuning in motorbridge.
DAMIAO_RW_REGISTERS: dict[int, RegisterSpec] = {
    0: RegisterSpec(0, "UV_Value", "Under-voltage protection value", "f32", "RW", "(10.0, 3.4E38]"),
    1: RegisterSpec(1, "KT_Value", "Torque coefficient", "f32", "RW", "[0.0, 3.4E38]"),
    2: RegisterSpec(2, "OT_Value", "Over-temperature protection value", "f32", "RW", "[80.0, 200)"),
    3: RegisterSpec(3, "OC_Value", "Over-current protection value", "f32", "RW", "(0.0, 1.0)"),
    4: RegisterSpec(4, "ACC", "Acceleration", "f32", "RW", "(0.0, 3.4E38)"),
    5: RegisterSpec(5, "DEC", "Deceleration", "f32", "RW", "[-3.4E38, 0.0)"),
    6: RegisterSpec(6, "MAX_SPD", "Maximum speed", "f32", "RW", "(0.0, 3.4E38]"),
    7: RegisterSpec(7, "MST_ID", "Feedback ID", "u32", "RW", "[0, 0x7FF]"),
    8: RegisterSpec(8, "ESC_ID", "Receive ID", "u32", "RW", "[0, 0x7FF]"),
    9: RegisterSpec(9, "TIMEOUT", "Timeout alarm time", "u32", "RW", "[0, 2^32-1]"),
    10: RegisterSpec(10, "CTRL_MODE", "Control mode", "u32", "RW", "[1, 4]"),
    21: RegisterSpec(21, "PMAX", "Position mapping range", "f32", "RW", "(0.0, 3.4E38]"),
    22: RegisterSpec(22, "VMAX", "Speed mapping range", "f32", "RW", "(0.0, 3.4E38]"),
    23: RegisterSpec(23, "TMAX", "Torque mapping range", "f32", "RW", "(0.0, 3.4E38]"),
    24: RegisterSpec(24, "I_BW", "Current loop control bandwidth", "f32", "RW", "[100.0, 10000.0]"),
    25: RegisterSpec(25, "KP_ASR", "Speed loop Kp", "f32", "RW", "[0.0, 3.4E38]"),
    26: RegisterSpec(26, "KI_ASR", "Speed loop Ki", "f32", "RW", "[0.0, 3.4E38]"),
    27: RegisterSpec(27, "KP_APR", "Position loop Kp", "f32", "RW", "[0.0, 3.4E38]"),
    28: RegisterSpec(28, "KI_APR", "Position loop Ki", "f32", "RW", "[0.0, 3.4E38]"),
    29: RegisterSpec(29, "OV_Value", "Over-voltage protection value", "f32", "RW", "TBD"),
    30: RegisterSpec(30, "GREF", "Gear torque efficiency", "f32", "RW", "(0.0, 1.0]"),
    31: RegisterSpec(31, "Deta", "Speed loop damping coefficient", "f32", "RW", "[1.0, 30.0]"),
    32: RegisterSpec(32, "V_BW", "Speed loop filter bandwidth", "f32", "RW", "(0.0, 500.0)"),
    33: RegisterSpec(33, "IQ_c1", "Current loop enhancement coefficient", "f32", "RW", "[100.0, 10000.0]"),
    34: RegisterSpec(34, "VL_c1", "Speed loop enhancement coefficient", "f32", "RW", "(0.0, 10000.0]"),
    35: RegisterSpec(35, "can_br", "CAN baud rate code", "u32", "RW", "[0, 4]"),
}

DAMIAO_HIGH_IMPACT_RIDS: tuple[int, ...] = (21, 22, 23, 25, 26, 27, 28, 4, 5, 6, 9)
DAMIAO_PROTECTION_RIDS: tuple[int, ...] = (0, 2, 3, 29)

RID_CTRL_MODE = 10
RID_MST_ID = 7
RID_ESC_ID = 8
RID_TIMEOUT = 9
RID_PMAX = 21
RID_VMAX = 22
RID_TMAX = 23
RID_KP_ASR = 25
RID_KI_ASR = 26
RID_KP_APR = 27
RID_KI_APR = 28

MODE_MIT = 1
MODE_POS_VEL = 2
MODE_VEL = 3
MODE_FORCE_POS = 4


def get_damiao_register_spec(rid: int) -> RegisterSpec | None:
    return DAMIAO_RW_REGISTERS.get(rid)
