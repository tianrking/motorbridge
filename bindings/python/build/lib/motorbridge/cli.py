from __future__ import annotations

import argparse
import time

from .core import Controller
from .models import Mode


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="motorbridge Python SDK CLI")
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4340")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")
    p.add_argument("--mode", default="mit", choices=["enable", "disable", "mit", "pos-vel", "vel", "force-pos"])
    p.add_argument("--loop", type=int, default=100)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--ensure-mode", type=int, default=1)
    p.add_argument("--ensure-strict", type=int, default=0)
    p.add_argument("--ensure-timeout-ms", type=int, default=1000)
    p.add_argument("--print-state", type=int, default=1)

    p.add_argument("--pos", type=float, default=0.0)
    p.add_argument("--vel", type=float, default=0.0)
    p.add_argument("--kp", type=float, default=30.0)
    p.add_argument("--kd", type=float, default=1.0)
    p.add_argument("--tau", type=float, default=0.0)
    p.add_argument("--vlim", type=float, default=1.0)
    p.add_argument("--ratio", type=float, default=0.3)
    return p.parse_args()


def _mode_to_enum(mode: str) -> Mode:
    return {
        "mit": Mode.MIT,
        "pos-vel": Mode.POS_VEL,
        "vel": Mode.VEL,
        "force-pos": Mode.FORCE_POS,
    }[mode]


def main() -> None:
    args = parse_args()
    motor_id = int(args.motor_id, 0)
    feedback_id = int(args.feedback_id, 0)

    print(
        f"channel={args.channel} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} mode={args.mode}"
    )

    with Controller(args.channel) as ctrl:
        motor = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            if args.mode not in ("enable", "disable"):
                ctrl.enable_all()
                time.sleep(0.3)

            if args.ensure_mode and args.mode not in ("enable", "disable"):
                try:
                    motor.ensure_mode(_mode_to_enum(args.mode), args.ensure_timeout_ms)
                except Exception as e:
                    if args.ensure_strict:
                        raise
                    print(f"[warn] ensure_mode failed: {e}; continue anyway")

            for i in range(args.loop):
                if args.mode == "enable":
                    motor.enable()
                    motor.request_feedback()
                elif args.mode == "disable":
                    motor.disable()
                    motor.request_feedback()
                elif args.mode == "mit":
                    motor.send_mit(args.pos, args.vel, args.kp, args.kd, args.tau)
                elif args.mode == "pos-vel":
                    motor.send_pos_vel(args.pos, args.vlim)
                elif args.mode == "vel":
                    motor.send_vel(args.vel)
                elif args.mode == "force-pos":
                    motor.send_force_pos(args.pos, args.vlim, args.ratio)

                if args.print_state:
                    st = motor.get_state()
                    if st is None:
                        print(f"#{i} no feedback yet")
                    else:
                        print(
                            f"#{i} pos={st.pos:+.3f} vel={st.vel:+.3f} "
                            f"torq={st.torq:+.3f} status={st.status_code}"
                        )
                time.sleep(max(args.dt_ms, 0) / 1000.0)
        finally:
            motor.close()


if __name__ == "__main__":
    main()
