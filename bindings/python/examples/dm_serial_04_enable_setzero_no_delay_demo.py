#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(
        description=(
            "SOP-04 (dm-serial): enable -> set_zero_position -> immediate control "
            "(default no delay between steps)"
        )
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x04")
    p.add_argument("--feedback-id", default="0x14")
    p.add_argument("--ensure-timeout-ms", type=int, default=300)
    p.add_argument("--loop", type=int, default=50)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--target-pos", type=float, default=0.0, help="pos-vel target position (rad)")
    p.add_argument("--vlim", type=float, default=1.0, help="pos-vel velocity limit (rad/s)")
    p.add_argument(
        "--settle-ms",
        type=int,
        default=0,
        help="optional delay after set_zero_position (default 0 for stress test)",
    )
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    loop_n = max(1, int(args.loop))
    dt_s = max(0, int(args.dt_ms)) / 1000.0
    settle_s = max(0, int(args.settle_ms)) / 1000.0

    print(
        f"sop=04-enable-setzero-no-delay serial={args.serial_port}@{args.serial_baud} "
        f"model={args.model} motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} "
        f"loop={loop_n} dt_ms={args.dt_ms} target_pos={args.target_pos} vlim={args.vlim} "
        f"settle_ms={args.settle_ms}"
    )

    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            # 1) Enable.
            ctrl.enable_all()

            # 2) Set current position as zero.
            m.set_zero_position()

            # Optional settle time for A/B comparison.
            if settle_s > 0:
                time.sleep(settle_s)

            # 3) Immediately switch mode and run control loop.
            # m.ensure_mode(Mode.POS_VEL, int(args.ensure_timeout_ms))
            # for i in range(loop_n):
            #     m.send_pos_vel(float(args.target_pos), float(args.vlim))
            #     try:
            #         m.request_feedback()
            #         ctrl.poll_feedback_once()
            #     except Exception:
            #         pass

            #     st = m.get_state()
            #     if st is None:
            #         print(f"#{i+1}/{loop_n} no_feedback")
            #     else:
            #         print(
            #             f"#{i+1}/{loop_n} pos={st.pos:+.3f} vel={st.vel:+.3f} "
            #             f"torq={st.torq:+.3f} status={st.status_code}"
            #         )

            #     if dt_s > 0:
            #         time.sleep(dt_s)
        finally:
            try:
                # m.disable()
                pass
            except Exception:
                pass
            # m.close()
            pass


if __name__ == "__main__":
    main()

