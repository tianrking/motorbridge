#!/usr/bin/env python3
"""08: mixed mode switching in one run.

Sequence:
- POS_VEL hold
- MIT hold
- back to POS_VEL
"""

from __future__ import annotations

import time

from motorbridge import Controller, Mode

CHANNEL = "can0"
VENDOR = "damiao"
MODEL = "4340P"
MOTOR_ID = 0x01
FEEDBACK_ID = 0x11
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


def run_loop(loop: int, dt_s: float, fn):
    for i in range(loop):
        t0 = time.time()
        fn(i)
        rest = dt_s - (time.time() - t0)
        if rest > 0:
            time.sleep(rest)


def main() -> int:
    dt_s = max(DT_MS, 1) / 1000.0
    with Controller(CHANNEL) as ctrl:
        m = add_motor(ctrl)
        ctrl.enable_all()

        m.ensure_mode(Mode.POS_VEL, 1000)
        run_loop(40, dt_s, lambda i: (m.send_pos_vel(0.8, 1.0), print(f"A#{i} {m.get_state()}")))

        m.ensure_mode(Mode.MIT, 1000)
        run_loop(40, dt_s, lambda i: (m.send_mit(0.8, 0.0, 14.0, 0.8, 0.0), print(f"B#{i} {m.get_state()}")))

        m.ensure_mode(Mode.POS_VEL, 1000)
        run_loop(40, dt_s, lambda i: (m.send_pos_vel(0.0, 1.0), print(f"C#{i} {m.get_state()}")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
