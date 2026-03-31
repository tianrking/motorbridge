#!/usr/bin/env python3
"""Quickstart 02 (super simple): control ONE motor.

How to use:
1) Change the constants below to your real IDs.
2) Run: python3 quickstart_02_single_motor.py
"""

from __future__ import annotations

import time

from motorbridge import Controller, Mode

# ----- edit these lines -----
TRANSPORT = "auto"  # auto / socketcan / dm-serial
CHANNEL = "can0"  # Linux: can0/slcan0, Windows PCAN: can0@1000000
MOTOR_ID = 0x01
FEEDBACK_ID = 0x11
MODEL = "4340P"
TARGET_POS = 1.0  # rad
V_LIMIT = 1.0  # rad/s
LOOP = 80
DT_MS = 20
SERIAL_PORT = "/dev/ttyACM0"  # dm-serial only
SERIAL_BAUD = 921600  # dm-serial only
# ---------------------------


def main() -> int:
    dt_s = max(DT_MS, 1) / 1000.0

    # Clear transport behavior:
    # - dm-serial is Damiao-only and uses serial_port/baud
    # - otherwise use normal CAN channel
    if TRANSPORT == "dm-serial":
        ctrl = Controller.from_dm_serial(SERIAL_PORT, SERIAL_BAUD)
    else:
        ctrl = Controller(CHANNEL)

    with ctrl:
        motor = ctrl.add_damiao_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
        ctrl.enable_all()
        motor.ensure_mode(Mode.POS_VEL, 1000)

        for i in range(LOOP):
            t0 = time.time()
            motor.send_pos_vel(TARGET_POS, V_LIMIT)
            print(f"#{i} state={motor.get_state()}")
            sleep_s = dt_s - (time.time() - t0)
            if sleep_s > 0:
                time.sleep(sleep_s)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
