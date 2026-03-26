#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def _mode_to_enum(mode: str) -> Mode:
    return {
        "mit": Mode.MIT,
        "pos-vel": Mode.POS_VEL,
        "vel": Mode.VEL,
        "force-pos": Mode.FORCE_POS,
    }[mode]


def main() -> None:
    p = argparse.ArgumentParser(
        description="SOP-02 (dm-serial): normal control loop without calibration/config writes"
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x04")
    p.add_argument("--feedback-id", default="0x14")
    p.add_argument("--mode", choices=["mit", "pos-vel", "vel", "force-pos"], default="mit")
    p.add_argument("--ensure-timeout-ms", type=int, default=300)
    p.add_argument("--loop", type=int, default=200)
    p.add_argument("--dt-ms", type=int, default=20)

    p.add_argument("--pos", type=float, default=0.0)
    p.add_argument("--vel", type=float, default=0.0)
    p.add_argument("--kp", type=float, default=2.0)
    p.add_argument("--kd", type=float, default=0.2)
    p.add_argument("--tau", type=float, default=0.0)
    p.add_argument("--vlim", type=float, default=1.0)
    p.add_argument("--ratio", type=float, default=0.2)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    loop_n = max(1, int(args.loop))
    dt_s = max(0, int(args.dt_ms)) / 1000.0

    print(
        f"sop=02-control serial={args.serial_port}@{args.serial_baud} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} "
        f"mode={args.mode} loop={loop_n} dt_ms={args.dt_ms}"
    )
    print("note=no calibration/config writes in this demo (no set_zero_position/set_can_timeout)")

    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            ctrl.enable_all()
            time.sleep(0.2)

            m.ensure_mode(_mode_to_enum(args.mode), int(args.ensure_timeout_ms))

            for i in range(loop_n):
                if args.mode == "mit":
                    m.send_mit(args.pos, args.vel, args.kp, args.kd, args.tau)
                elif args.mode == "pos-vel":
                    m.send_pos_vel(args.pos, args.vlim)
                elif args.mode == "vel":
                    m.send_vel(args.vel)
                else:
                    m.send_force_pos(args.pos, args.vlim, args.ratio)

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
                        f"torq={st.torq:+.3f} status={st.status_code}"
                    )

                if dt_s > 0:
                    time.sleep(dt_s)
        finally:
            try:
                m.disable()
            except Exception:
                pass
            m.close()


if __name__ == "__main__":
    main()

