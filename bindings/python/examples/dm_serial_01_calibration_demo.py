#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(
        description="SOP-01 (dm-serial): maintenance/calibration flow"
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x04")
    p.add_argument("--feedback-id", default="0x14")
    p.add_argument("--can-timeout-ms", type=int, default=1000)
    p.add_argument("--set-zero", type=int, default=0, help="1 to set current position as zero")
    p.add_argument("--settle-ms", type=int, default=200, help="wait after key ops")
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    settle_s = max(0, int(args.settle_ms)) / 1000.0

    print(
        f"sop=01-calibration serial={args.serial_port}@{args.serial_baud} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} "
        f"set_zero={args.set_zero} can_timeout_ms={args.can_timeout_ms}"
    )

    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            m.clear_error()
            print("clear_error: ok")

            m.set_can_timeout_ms(int(args.can_timeout_ms))
            print(f"set_can_timeout_ms: {args.can_timeout_ms}")

            if args.set_zero:
                m.set_zero_position()
                print("set_zero_position: command sent")
                if settle_s > 0:
                    time.sleep(settle_s)

            try:
                m.request_feedback()
                ctrl.poll_feedback_once()
            except Exception:
                pass

            st = m.get_state()
            if st is None:
                print("state: no feedback yet")
            else:
                print(
                    f"state pos={st.pos:+.3f} vel={st.vel:+.3f} torq={st.torq:+.3f} "
                    f"status={st.status_code} t_mos={st.t_mos:.1f}C t_rotor={st.t_rotor:.1f}C"
                )
        finally:
            m.close()


if __name__ == "__main__":
    main()

