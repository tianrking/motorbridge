#!/usr/bin/env python3
"""Simple 4-motor control demo (2 Damiao + 1 MyActuator + 1 RobStride).

Goal:
- provide a minimal multi-vendor reference with safe send rate,
- drive all motors with one shared swing target (+pos / -pos),
- keep RobStride direction configurable via rs_dir_sign.

Important beginner notes:
- One Controller cannot mix vendor families in this SDK.
- This file intentionally creates three controllers:
  Damiao group / MyActuator group / RobStride group.
- Keep sending rate conservative (`--dt-ms 20` or larger).
"""
import argparse
import time

from motorbridge import Controller, Mode
from motorbridge.errors import CallError


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Simple 4-motor control demo with safe send rate")
    p.add_argument(
        "--channel",
        default="can0",
        help="SocketCAN interface name (can0/can1/slcan0). Default: can0",
    )
    p.add_argument(
        "--loop",
        type=int,
        default=240,
        help="Total control iterations. Default: 240",
    )
    p.add_argument(
        "--dt-ms",
        type=int,
        default=20,
        help=(
            "Loop period in milliseconds. Keep >=20ms in mixed-vendor setup. "
            "If os error 105 appears, try 30 or 50."
        ),
    )
    p.add_argument(
        "--pos",
        type=float,
        default=1.0,
        help="Swing amplitude in radians for Damiao/MyActuator. Default: 1.0",
    )
    p.add_argument(
        "--swing-loop",
        type=int,
        default=60,
        help="Toggle between +pos and -pos every N iterations. Default: 60",
    )
    p.add_argument(
        "--rs-dir-sign",
        type=float,
        default=-1.0,
        help="RobStride direction sign (+1 same direction, -1 opposite). Default: -1",
    )
    p.add_argument(
        "--dm-vlim",
        type=float,
        default=1.0,
        help="Damiao POS_VEL speed limit. Default: 1.0",
    )
    p.add_argument(
        "--my-vlim",
        type=float,
        default=1.2,
        help="MyActuator POS_VEL speed limit. Default: 1.2",
    )
    p.add_argument(
        "--rs-kp",
        type=float,
        default=1.2,
        help="RobStride MIT Kp. Increase slowly if response is too soft. Default: 1.2",
    )
    p.add_argument(
        "--rs-kd",
        type=float,
        default=1.5,
        help="RobStride MIT Kd (damping). Default: 1.5",
    )
    return p.parse_args()


def main() -> None:
    args = parse_args()
    dt_s = max(args.dt_ms, 1) / 1000.0

    # Important: one Controller cannot mix vendors.
    # We create one controller per vendor group.
    with Controller(args.channel) as dm_ctrl, \
         Controller(args.channel) as my_ctrl, \
         Controller(args.channel) as rs_ctrl:

        # Register motors with (motor_id, feedback_id, model).
        # Adjust these IDs to your real hardware scan results.
        dm1 = dm_ctrl.add_damiao_motor(0x01, 0x11, "4340P")
        dm2 = dm_ctrl.add_damiao_motor(0x07, 0x17, "4310")
        mym = my_ctrl.add_myactuator_motor(1, 0x241, "X8")
        rsm = rs_ctrl.add_robstride_motor(127, 0xFE, "rs-00")

        # Enable all outputs.
        dm_ctrl.enable_all()
        my_ctrl.enable_all()
        rs_ctrl.enable_all()

        # Ensure expected mode per motor type.
        dm1.ensure_mode(Mode.POS_VEL, 1000)
        dm2.ensure_mode(Mode.POS_VEL, 1000)
        mym.ensure_mode(Mode.POS_VEL, 1000)
        rsm.ensure_mode(Mode.MIT, 1000)

        # Fixed-rate loop with alternating target for swing motion.
        # target sequence example when pos=1.0 and swing-loop=60:
        # +1.0 (60 loops) -> -1.0 (60 loops) -> +1.0 ...
        for i in range(args.loop):
            t0 = time.time()
            target = args.pos if ((i // max(args.swing_loop, 1)) % 2 == 0) else -args.pos
            rs_target = args.rs_dir_sign * target
            try:
                dm1.send_pos_vel(target, args.dm_vlim)
                dm2.send_pos_vel(target, args.dm_vlim)
                mym.send_pos_vel(target, args.my_vlim)
                rsm.send_mit(rs_target, 0.0, args.rs_kp, args.rs_kd, 0.0)
            except CallError as e:
                print(f"#{i} send error: {e}")
                break

            print(
                f"#{i} target={target:+.3f} rs_target={rs_target:+.3f} "
                f"dm1={dm1.get_state()} dm2={dm2.get_state()} "
                f"my={mym.get_state()} rs={rsm.get_state()}"
            )

            # Sleep to keep bus load predictable and avoid os error 105.
            # This is critical: without throttling, TX queue can overflow quickly.
            elapsed = time.time() - t0
            if elapsed < dt_s:
                time.sleep(dt_s - elapsed)


if __name__ == "__main__":
    main()
