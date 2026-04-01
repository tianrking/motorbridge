#!/usr/bin/env python3
"""00: enable + status query.

Goal:
- connect controller
- create one motor handle
- enable output
- request/get state
"""

from __future__ import annotations

import time

from motorbridge import Controller

# ===== USER CONFIG =====
TRANSPORT = "socketcan"  # socketcan / dm-serial / socketcanfd
CHANNEL = "can0"
SERIAL_PORT = "/dev/ttyACM0"
SERIAL_BAUD = 921600

VENDOR = "damiao"  # damiao/myactuator/robstride/hightorque/hexfellow
MODEL = "4310"
MOTOR_ID = 0x04
FEEDBACK_ID = 0x14

# Feedback retry policy (avoid one-shot None)
MAX_STATE_TRIES = 12
RETRY_DT_MS = 50
# =======================


def new_controller() -> Controller:
    if TRANSPORT == "dm-serial":
        return Controller.from_dm_serial(SERIAL_PORT, SERIAL_BAUD)
    if TRANSPORT == "socketcanfd":
        return Controller.from_socketcanfd(CHANNEL)
    return Controller(CHANNEL)


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
    with new_controller() as ctrl:
        motor = add_motor(ctrl)
        ctrl.enable_all()
        state = None
        for _ in range(MAX_STATE_TRIES):
            motor.request_feedback()
            ctrl.poll_feedback_once()  # <= v0.1.6: required. v0.1.7+: optional (background poll is default).
            state = motor.get_state()
            if state is not None:
                break
            time.sleep(max(RETRY_DT_MS, 1) / 1000.0)

        if state is None:
            print(
                "state=None (no fresh feedback yet). "
                "tip: run 03-mode-switch-method.py once, then retry this script."
            )
        else:
            print("state=", state)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
