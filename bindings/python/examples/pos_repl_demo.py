#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(description="Interactive position console (POS_VEL mode)")
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")
    p.add_argument("--vlim", type=float, default=1.5)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--ensure-timeout-ms", type=int, default=1000)
    p.add_argument("--print-state", type=int, default=1)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)

    print(
        f"channel={args.channel} model={args.model} motor_id=0x{motor_id:X} "
        f"feedback_id=0x{feedback_id:X} mode=pos-vel(repl) vlim={args.vlim}"
    )

    with Controller(args.channel) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            ctrl.enable_all()
            time.sleep(0.3)
            m.ensure_mode(Mode.POS_VEL, args.ensure_timeout_ms)

            print("ready. input target pos (rad), or q to quit")
            while True:
                try:
                    text = input("> ").strip()
                except EOFError:
                    break
                if not text:
                    continue
                if text in {"q", "quit", "exit"}:
                    break
                if text == "help":
                    print("input example: 1 / 3.14 ; commands: q|quit|exit|help")
                    continue

                try:
                    target = float(text)
                except ValueError:
                    print(f"invalid input: {text}")
                    continue

                m.send_pos_vel(target, args.vlim)
                if args.dt_ms > 0:
                    time.sleep(args.dt_ms / 1000.0)
                if args.print_state:
                    st = m.get_state()
                    if st is None:
                        print(f"target={target} (no feedback yet)")
                    else:
                        print(
                            f"target={target:+.3f} pos={st.pos:+.3f} vel={st.vel:+.3f} "
                            f"torq={st.torq:+.3f} status={st.status_code}"
                        )
                else:
                    print(f"target sent: {target}")
        finally:
            m.close()

    print("bye")


if __name__ == "__main__":
    main()

