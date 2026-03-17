#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def _parse_mode(text: str) -> str:
    v = text.strip().lower()
    if v not in {"enable", "disable", "mit", "pos-vel", "vel", "force-pos"}:
        raise argparse.ArgumentTypeError(f"unknown mode: {text}")
    return v


def main() -> None:
    p = argparse.ArgumentParser(description="Python SDK multi-mode demo (full params)")
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4340P")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")
    p.add_argument("--mode", type=_parse_mode, default="mit")
    p.add_argument("--loop", type=int, default=100)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--ensure-mode", type=int, default=1)
    p.add_argument("--ensure-timeout-ms", type=int, default=1000)
    p.add_argument("--ensure-strict", type=int, default=0)
    p.add_argument("--print-state", type=int, default=1)

    p.add_argument("--pos", type=float, default=0.0)
    p.add_argument("--vel", type=float, default=0.0)
    p.add_argument("--kp", type=float, default=20.0)
    p.add_argument("--kd", type=float, default=1.0)
    p.add_argument("--tau", type=float, default=0.0)
    p.add_argument("--vlim", type=float, default=1.5)
    p.add_argument("--ratio", type=float, default=0.3)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)

    print(
        f"channel={args.channel} model={args.model} motor_id=0x{motor_id:X} "
        f"feedback_id=0x{feedback_id:X} mode={args.mode}"
    )

    with Controller(args.channel) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            if args.mode not in {"enable", "disable"}:
                ctrl.enable_all()
                time.sleep(0.3)

            if args.ensure_mode and args.mode not in {"enable", "disable"}:
                mode_map = {
                    "mit": Mode.MIT,
                    "pos-vel": Mode.POS_VEL,
                    "vel": Mode.VEL,
                    "force-pos": Mode.FORCE_POS,
                }
                try:
                    m.ensure_mode(mode_map[args.mode], args.ensure_timeout_ms)
                except Exception:
                    if args.ensure_strict:
                        raise
                    print("[warn] ensure_mode failed, continue anyway")

            for i in range(args.loop):
                if args.mode == "enable":
                    m.enable()
                    m.request_feedback()
                elif args.mode == "disable":
                    m.disable()
                    m.request_feedback()
                elif args.mode == "mit":
                    m.send_mit(args.pos, args.vel, args.kp, args.kd, args.tau)
                elif args.mode == "pos-vel":
                    m.send_pos_vel(args.pos, args.vlim)
                elif args.mode == "vel":
                    m.send_vel(args.vel)
                elif args.mode == "force-pos":
                    m.send_force_pos(args.pos, args.vlim, args.ratio)

                if args.print_state:
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
