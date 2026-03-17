#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(description="MIT loop demo aligned with C++ cpp_wrapper_demo")
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4340P")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")
    p.add_argument("--pos", type=float, default=0.0)
    p.add_argument("--vel", type=float, default=0.0)
    p.add_argument("--kp", type=float, default=20.0)
    p.add_argument("--kd", type=float, default=1.0)
    p.add_argument("--tau", type=float, default=0.0)
    p.add_argument("--loop", type=int, default=50)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--ensure-timeout-ms", type=int, default=1000)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)

    print(
        f"channel={args.channel} model={args.model} motor_id=0x{motor_id:X} "
        f"feedback_id=0x{feedback_id:X} mode=mit"
    )

    with Controller(args.channel) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            ctrl.enable_all()
            m.ensure_mode(Mode.MIT, args.ensure_timeout_ms)

            for i in range(args.loop):
                m.send_mit(args.pos, args.vel, args.kp, args.kd, args.tau)
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
