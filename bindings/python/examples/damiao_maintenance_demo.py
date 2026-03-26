#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(
        description="Damiao maintenance demo: clear-error / set-zero / timeout / feedback"
    )
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4340P")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")
    p.add_argument("--can-timeout-ms", type=int, default=1000)
    p.add_argument("--set-zero", type=int, default=0, help="1 to set zero position")
    p.add_argument("--disable-at-end", type=int, default=1, help="1 to disable at end")
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)

    print(
        f"channel={args.channel} model={args.model} motor_id=0x{motor_id:X} "
        f"feedback_id=0x{feedback_id:X}"
    )

    with Controller(args.channel) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            ctrl.enable_all()
            time.sleep(0.2)

            m.set_can_timeout_ms(args.can_timeout_ms)
            print(f"set_can_timeout_ms={args.can_timeout_ms}")

            m.clear_error()
            print("clear_error done")

            if args.set_zero:
                try:
                    m.disable()
                except Exception:
                    pass
                m.set_zero_position()
                print("set_zero_position done")
                time.sleep(0.2)

            m.request_feedback()
            st = m.get_state()
            if st is None:
                print("state: no feedback yet")
            else:
                print(
                    f"state pos={st.pos:+.3f} vel={st.vel:+.3f} torq={st.torq:+.3f} "
                    f"status={st.status_code} t_mos={st.t_mos:.1f} t_rotor={st.t_rotor:.1f}"
                )
        finally:
            if args.disable_at_end:
                m.disable()
            m.close()


if __name__ == "__main__":
    main()
