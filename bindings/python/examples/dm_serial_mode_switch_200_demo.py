#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(
        description="Damiao dm-serial mode switch proof demo (MIT -> POS_VEL -> VEL -> FORCE_POS)"
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x07")
    p.add_argument("--feedback-id", default="0x17")
    p.add_argument("--ensure-timeout-ms", type=int, default=300)
    p.add_argument("--loop-per-mode", type=int, default=200)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--mode-gap-ms", type=int, default=300)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    loops = max(1, int(args.loop_per_mode))
    dt_s = max(0, int(args.dt_ms)) / 1000.0
    gap_s = max(0, int(args.mode_gap_ms)) / 1000.0

    print(
        f"demo=dm-serial-mode-switch serial={args.serial_port}@{args.serial_baud} "
        f"model={args.model} motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} "
        f"ensure_timeout_ms={args.ensure_timeout_ms} loop_per_mode={loops} dt_ms={args.dt_ms}"
    )

    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        motor = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            ctrl.enable_all()
            time.sleep(0.2)

            # Each step is: (display_name, mode_enum, send_command_lambda)
            # The command values are intentionally conservative for dm-serial stability.
            steps = [
                # MIT: pos, vel, kp, kd, tau
                ("MIT", Mode.MIT, lambda: motor.send_mit(-3, 0.0, 2.0, 0.2, 0.0)),
                # POS_VEL: target position(rad), velocity limit(rad/s)
                ("POS_VEL", Mode.POS_VEL, lambda: motor.send_pos_vel(0.3, 0.8)),
                # VEL: target velocity(rad/s)
                ("VEL", Mode.VEL, lambda: motor.send_vel(0.5)),
                # FORCE_POS: target position(rad), velocity limit(rad/s), torque limit ratio(0~1)
                ("FORCE_POS", Mode.FORCE_POS, lambda: motor.send_force_pos(0.3, 0.8, 0.2)),
            ]

            for name, mode, send_cmd in steps:
                # Explicitly switch control mode and verify success before sending commands.
                print(f"\n=== switch -> {name} ===")
                motor.ensure_mode(mode, args.ensure_timeout_ms)
                print(f"{name}: ensure_mode ok, running {loops} loops")

                for i in range(loops):
                    # Send one control command for the current mode.
                    send_cmd()
                    # Request and poll one feedback cycle for observability.
                    motor.request_feedback()
                    try:
                        ctrl.poll_feedback_once()
                    except Exception:
                        pass
                    st = motor.get_state()
                    if st is None:
                        print(f"{name} #{i+1}/{loops}: no feedback yet")
                    else:
                        print(
                            f"{name} #{i+1}/{loops}: pos={st.pos:+.3f} vel={st.vel:+.3f} "
                            f"torq={st.torq:+.3f} status={st.status_code}"
                        )
                    if dt_s > 0:
                        time.sleep(dt_s)

                if gap_s > 0:
                    time.sleep(gap_s)
        finally:
            try:
                motor.disable()
            finally:
                motor.close()


if __name__ == "__main__":
    main()
