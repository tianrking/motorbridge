#!/usr/bin/env python3
"""03: common method for mode switching.

This lesson only focuses on ensure_mode() workflow.
"""

from __future__ import annotations

from motorbridge import Controller, Mode

# ===== USER CONFIG =====
CHANNEL = "can0"
VENDOR = "damiao"
MODEL = "4310"
MOTOR_ID = 0x04
FEEDBACK_ID = 0x14
MODE_NAME = "pos_vel"  # mit/pos_vel/vel/force_pos
# =======================


def add_motor(ctrl: Controller):
    if VENDOR == "damiao":
        return ctrl.add_damiao_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    if VENDOR == "myactuator":
        return ctrl.add_myactuator_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    if VENDOR == "robstride":
        return ctrl.add_robstride_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    if VENDOR == "hightorque":
        return ctrl.add_hightorque_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    if VENDOR == "hexfellow":
        return ctrl.add_hexfellow_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    raise ValueError(f"unsupported vendor: {VENDOR}")


def parse_mode(name: str) -> Mode:
    mapping = {
        "mit": Mode.MIT,
        "pos_vel": Mode.POS_VEL,
        "vel": Mode.VEL,
        "force_pos": Mode.FORCE_POS,
    }
    if name not in mapping:
        raise ValueError(f"bad MODE_NAME: {name}")
    return mapping[name]


def main() -> int:
    with Controller(CHANNEL) as ctrl:
        motor = add_motor(ctrl)
        ctrl.enable_all()
        motor.ensure_mode(parse_mode(MODE_NAME), 1000)
        print("mode switched to", MODE_NAME)
        print("state=", motor.get_state())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
