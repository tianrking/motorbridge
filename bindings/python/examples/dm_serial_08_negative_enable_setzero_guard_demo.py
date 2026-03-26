#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(
        description=(
            "SOP-08 negative test (dm-serial): intentionally call set_zero_position() "
            "while enabled, expect core guard to reject."
        )
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x04")
    p.add_argument("--feedback-id", default="0x14")
    p.add_argument("--ensure-timeout-ms", type=int, default=800)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)

    print(
        f"sop=08-negative-enable-setzero-guard serial={args.serial_port}@{args.serial_baud} "
        f"model={args.model} motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X}"
    )
    print("step=1 disable (clean start)")
    print("step=2 enable + ensure_mode(pos-vel)")
    print("step=3 call set_zero_position() in enabled state (expected reject)")

    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            try:
                m.disable()
            except Exception:
                pass

            ctrl.enable_all()
            m.ensure_mode(Mode.POS_VEL, int(args.ensure_timeout_ms))

            try:
                m.set_zero_position()
            except Exception as exc:
                print(f"PASS guard_triggered error={exc}")
                return

            print(
                "FAIL guard_not_triggered set_zero_position() succeeded while enabled; "
                "expected rejection"
            )
            sys.exit(2)
        finally:
            try:
                m.disable()
            except Exception:
                pass
            m.close()


if __name__ == "__main__":
    main()
