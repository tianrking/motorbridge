#!/usr/bin/env python3
"""Quickstart 03 (simple): control 4 motors together.

Why 3 controllers?
- one controller = one vendor family in this SDK.

Note:
- This mixed-vendor demo uses normal CAN channel path.
- dm-serial is Damiao-only, so it is not used in this 4-vendor example.
"""

from __future__ import annotations

import time

from motorbridge import Controller, Mode

# ----- edit these lines -----
CHANNEL = "can0"  # Linux: can0/slcan0, Windows PCAN: can0@1000000
LOOP = 120
DT_MS = 20
POS = 1.0  # swing amplitude
SWING_LOOP = 40  # change +POS / -POS every N loops

# Damiao
DM1_ID, DM1_FID, DM1_MODEL = 0x01, 0x11, "4340P"
DM2_ID, DM2_FID, DM2_MODEL = 0x07, 0x17, "4310"

# MyActuator
MY_ID, MY_FID, MY_MODEL = 1, 0x241, "X8"

# RobStride
RS_ID, RS_FID, RS_MODEL = 127, 0xFE, "rs-00"
RS_DIR_SIGN = -1.0  # set +1.0 if same direction as others
# ---------------------------


def main() -> int:
    dt_s = max(DT_MS, 1) / 1000.0

    with Controller(CHANNEL) as dm_ctrl, Controller(CHANNEL) as my_ctrl, Controller(CHANNEL) as rs_ctrl:
        dm1 = dm_ctrl.add_damiao_motor(DM1_ID, DM1_FID, DM1_MODEL)
        dm2 = dm_ctrl.add_damiao_motor(DM2_ID, DM2_FID, DM2_MODEL)
        mym = my_ctrl.add_myactuator_motor(MY_ID, MY_FID, MY_MODEL)
        rsm = rs_ctrl.add_robstride_motor(RS_ID, RS_FID, RS_MODEL)

        dm_ctrl.enable_all()
        my_ctrl.enable_all()
        rs_ctrl.enable_all()

        dm1.ensure_mode(Mode.POS_VEL, 1000)
        dm2.ensure_mode(Mode.POS_VEL, 1000)
        mym.ensure_mode(Mode.POS_VEL, 1000)
        rsm.ensure_mode(Mode.MIT, 1000)

        for i in range(LOOP):
            t0 = time.time()
            target = POS if ((i // max(SWING_LOOP, 1)) % 2 == 0) else -POS
            dm1.send_pos_vel(target, 1.0)
            dm2.send_pos_vel(target, 1.0)
            mym.send_pos_vel(target, 1.2)
            rsm.send_mit(RS_DIR_SIGN * target, 0.0, 1.2, 1.5, 0.0)
            print(f"#{i} target={target:+.2f}")
            sleep_s = dt_s - (time.time() - t0)
            if sleep_s > 0:
                time.sleep(sleep_s)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
