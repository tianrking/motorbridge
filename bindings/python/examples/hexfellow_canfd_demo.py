#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(description="Hexfellow CAN-FD demo (MIT / POS_VEL only)")
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="hexfellow")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x00")
    p.add_argument("--mode", default="mit", choices=["mit", "pos-vel"])
    p.add_argument("--loop", type=int, default=20)
    p.add_argument("--dt-ms", type=int, default=50)
    p.add_argument("--ensure-timeout-ms", type=int, default=1000)
    p.add_argument("--pos", type=float, default=0.8)
    p.add_argument("--vel", type=float, default=1.0)
    p.add_argument("--vlim", type=float, default=1.0)
    p.add_argument("--kp", type=float, default=30.0)
    p.add_argument("--kd", type=float, default=1.0)
    p.add_argument("--tau", type=float, default=0.1)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)

    print(
        f"vendor=hexfellow transport=socketcanfd channel={args.channel} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} mode={args.mode}"
    )

    with Controller.from_socketcanfd(args.channel) as ctrl:
        motor = ctrl.add_hexfellow_motor(motor_id, feedback_id, args.model)
        try:
            ctrl.enable_all()
            if args.mode == "mit":
                motor.ensure_mode(Mode.MIT, args.ensure_timeout_ms)
            else:
                motor.ensure_mode(Mode.POS_VEL, args.ensure_timeout_ms)

            for i in range(args.loop):
                if args.mode == "mit":
                    motor.send_mit(args.pos, args.vel, args.kp, args.kd, args.tau)
                else:
                    motor.send_pos_vel(args.pos, args.vlim)
                motor.request_feedback()
                st = motor.get_state()
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
            motor.close()


if __name__ == "__main__":
    main()
