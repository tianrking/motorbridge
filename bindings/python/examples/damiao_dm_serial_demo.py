#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def _parse_mode(text: str) -> str:
    v = text.strip().lower()
    if v not in {"mit", "pos-vel", "vel", "force-pos"}:
        raise argparse.ArgumentTypeError(f"unknown mode: {text}")
    return v


def main() -> None:
    p = argparse.ArgumentParser(
        description="Damiao dm-serial transport demo (Controller.from_dm_serial)"
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")
    p.add_argument("--mode", type=_parse_mode, default="mit")
    p.add_argument("--loop", type=int, default=80)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--ensure-timeout-ms", type=int, default=1000)
    p.add_argument("--pos", type=float, default=0.0)
    p.add_argument("--vel", type=float, default=0.0)
    p.add_argument("--kp", type=float, default=20.0)
    p.add_argument("--kd", type=float, default=1.0)
    p.add_argument("--tau", type=float, default=0.0)
    p.add_argument("--vlim", type=float, default=1.0)
    p.add_argument("--ratio", type=float, default=0.2)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)

    print(
        f"transport=dm-serial serial={args.serial_port}@{args.serial_baud} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} mode={args.mode}"
    )

    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            ctrl.enable_all()
            time.sleep(0.2)

            mode_map = {
                "mit": Mode.MIT,
                "pos-vel": Mode.POS_VEL,
                "vel": Mode.VEL,
                "force-pos": Mode.FORCE_POS,
            }
            m.ensure_mode(mode_map[args.mode], args.ensure_timeout_ms)

            for i in range(args.loop):
                if args.mode == "mit":
                    m.send_mit(args.pos, args.vel, args.kp, args.kd, args.tau)
                elif args.mode == "pos-vel":
                    m.send_pos_vel(args.pos, args.vlim)
                elif args.mode == "vel":
                    m.send_vel(args.vel)
                elif args.mode == "force-pos":
                    m.send_force_pos(args.pos, args.vlim, args.ratio)

                st = m.get_state()
                if st is None:
                    print(f"#{i} no feedback yet")
                else:
                    print(
                        f"#{i} pos={st.pos:+.3f} vel={st.vel:+.3f} "
                        f"torq={st.torq:+.3f} status={st.status_code}"
                    )
                if args.dt_ms > 0:
                    time.sleep(args.dt_ms / 1000.0)
        finally:
            m.close()


if __name__ == "__main__":
    main()

