#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(
        description="SOP-03 (dm-serial): status-only monitor for one motor"
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x04")
    p.add_argument("--feedback-id", default="0x14")
    p.add_argument("--loop", type=int, default=100)
    p.add_argument("--dt-ms", type=int, default=50)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    loop_n = max(1, int(args.loop))
    dt_s = max(0, int(args.dt_ms)) / 1000.0

    print(
        f"sop=03-status serial={args.serial_port}@{args.serial_baud} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} loop={loop_n} dt_ms={args.dt_ms}"
    )

    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            for i in range(loop_n):
                try:
                    m.request_feedback()
                    ctrl.poll_feedback_once()
                except Exception:
                    pass

                st = m.get_state()
                if st is None:
                    print(f"#{i+1}/{loop_n} no_feedback")
                else:
                    print(
                        f"#{i+1}/{loop_n} pos={st.pos:+.3f} vel={st.vel:+.3f} "
                        f"torq={st.torq:+.3f} status={st.status_code} "
                        f"t_mos={st.t_mos:.1f}C t_rotor={st.t_rotor:.1f}C"
                    )

                if dt_s > 0:
                    time.sleep(dt_s)
        finally:
            m.close()


if __name__ == "__main__":
    main()

