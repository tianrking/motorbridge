#!/usr/bin/env python3
from __future__ import annotations

import argparse
import contextlib
import time

from motorbridge import Controller, Mode


def _parse_int(v: str) -> int:
    return int(v, 0)


def main() -> None:
    p = argparse.ArgumentParser(description="4-motor position sync via Python binding (no ws_gateway)")
    p.add_argument("--channel", default="can0")
    p.add_argument("--pos", type=float, default=0.0)
    p.add_argument("--loop", type=int, default=120)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--stagger-ms", type=int, default=8)

    p.add_argument("--dm1-model", default="4340P")
    p.add_argument("--dm1-id", default="0x01")
    p.add_argument("--dm1-fid", default="0x11")
    p.add_argument("--dm1-vlim", type=float, default=1.0)

    p.add_argument("--dm2-model", default="4310")
    p.add_argument("--dm2-id", default="0x07")
    p.add_argument("--dm2-fid", default="0x17")
    p.add_argument("--dm2-vlim", type=float, default=1.0)

    p.add_argument("--my-model", default="X8")
    p.add_argument("--my-id", default="1")
    p.add_argument("--my-fid", default="0x241")
    p.add_argument("--my-vlim", type=float, default=1.2)

    p.add_argument("--rs-model", default="rs-00")
    p.add_argument("--rs-id", default="127")
    p.add_argument("--rs-fid", default="0xFE")
    p.add_argument("--rs-kp", type=float, default=1.2)
    p.add_argument("--rs-kd", type=float, default=1.5)
    p.add_argument("--rs-vel", type=float, default=0.0)
    p.add_argument("--rs-tau", type=float, default=0.0)
    p.add_argument(
        "--rs-dir-sign",
        type=float,
        default=-1.0,
        help="Direction sign for RobStride target angle (default: -1.0)",
    )
    args = p.parse_args()

    dm1_id, dm1_fid = _parse_int(args.dm1_id), _parse_int(args.dm1_fid)
    dm2_id, dm2_fid = _parse_int(args.dm2_id), _parse_int(args.dm2_fid)
    my_id, my_fid = _parse_int(args.my_id), _parse_int(args.my_fid)
    rs_id, rs_fid = _parse_int(args.rs_id), _parse_int(args.rs_fid)

    print(
        f"[binding] channel={args.channel} pos={args.pos:.3f} loop={args.loop} "
        f"dt_ms={args.dt_ms} stagger_ms={args.stagger_ms} rs_dir_sign={args.rs_dir_sign:+.1f}"
    )
    with contextlib.ExitStack() as stack:
        # Important: current binding controller is vendor-bound.
        # Use one controller per vendor family.
        dm_ctrl = stack.enter_context(Controller(args.channel))
        my_ctrl = stack.enter_context(Controller(args.channel))
        rs_ctrl = stack.enter_context(Controller(args.channel))

        dm1 = dm_ctrl.add_damiao_motor(dm1_id, dm1_fid, args.dm1_model)
        dm2 = dm_ctrl.add_damiao_motor(dm2_id, dm2_fid, args.dm2_model)
        mym = my_ctrl.add_myactuator_motor(my_id, my_fid, args.my_model)
        rsm = rs_ctrl.add_robstride_motor(rs_id, rs_fid, args.rs_model)

        motors = [dm1, dm2, mym, rsm]
        try:
            dm_ctrl.enable_all()
            my_ctrl.enable_all()
            rs_ctrl.enable_all()

            dm1.ensure_mode(Mode.POS_VEL, 1000)
            dm2.ensure_mode(Mode.POS_VEL, 1000)
            mym.ensure_mode(Mode.POS_VEL, 1000)
            rsm.ensure_mode(Mode.MIT, 1000)

            for i in range(args.loop):
                dm1.send_pos_vel(args.pos, args.dm1_vlim)
                if args.stagger_ms > 0:
                    time.sleep(args.stagger_ms / 1000.0)
                dm2.send_pos_vel(args.pos, args.dm2_vlim)
                if args.stagger_ms > 0:
                    time.sleep(args.stagger_ms / 1000.0)
                mym.send_pos_vel(args.pos, args.my_vlim)
                if args.stagger_ms > 0:
                    time.sleep(args.stagger_ms / 1000.0)
                rs_pos = args.pos * args.rs_dir_sign
                rsm.send_mit(rs_pos, args.rs_vel, args.rs_kp, args.rs_kd, args.rs_tau)

                if i % max(1, int(200 / max(1, args.dt_ms))) == 0:
                    print(f"#{i} dm1={dm1.get_state()} dm2={dm2.get_state()} my={mym.get_state()} rs={rsm.get_state()}")
                if args.dt_ms > 0:
                    time.sleep(args.dt_ms / 1000.0)
        finally:
            for m in motors:
                try:
                    m.close()
                except Exception:
                    pass


if __name__ == "__main__":
    main()
