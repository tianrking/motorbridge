#!/usr/bin/env python3
"""07: FORCE_POS mode standalone lesson."""

from __future__ import annotations

import time

from motorbridge import Controller, Mode

CHANNEL = "can0"
VENDOR = "damiao"
MODEL = "4340P"
MOTOR_ID = 0x01
FEEDBACK_ID = 0x11

FP_POS = 1.0
FP_VLIM = 1.0
FP_RATIO = 0.5
LOOP = 80
DT_MS = 20


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


def main() -> int:
    dt_s = max(DT_MS, 1) / 1000.0
    with Controller(CHANNEL) as ctrl:
        m = add_motor(ctrl)
        ctrl.enable_all()
        m.ensure_mode(Mode.FORCE_POS, 1000)
        for i in range(LOOP):
            t0 = time.time()
            m.send_force_pos(FP_POS, FP_VLIM, FP_RATIO)
            print(f"#{i} {m.get_state()}")
            rest = dt_s - (time.time() - t0)
            if rest > 0:
                time.sleep(rest)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
