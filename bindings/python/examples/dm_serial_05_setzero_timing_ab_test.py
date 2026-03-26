#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def run_one_case(
    serial_port: str,
    serial_baud: int,
    model: str,
    motor_id: int,
    feedback_id: int,
    settle_ms: int,
    rounds: int,
    ensure_timeout_ms: int,
) -> tuple[int, int]:
    ok = 0
    fail = 0

    for _ in range(rounds):
        with Controller.from_dm_serial(serial_port, serial_baud) as ctrl:
            m = ctrl.add_damiao_motor(motor_id, feedback_id, model)
            try:
                ctrl.enable_all()
                m.set_zero_position()
                if settle_ms > 0:
                    time.sleep(settle_ms / 1000.0)
                m.ensure_mode(Mode.POS_VEL, ensure_timeout_ms)
                m.send_pos_vel(0.0, 1.0)
                ok += 1
            except Exception:
                fail += 1
            finally:
                try:
                    m.disable()
                except Exception:
                    pass
                m.close()
    return ok, fail


def main() -> None:
    p = argparse.ArgumentParser(
        description="SOP-05 (dm-serial): A/B test for set_zero_position settle time"
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x04")
    p.add_argument("--feedback-id", default="0x14")
    p.add_argument("--settle-list-ms", default="0,50,100,200")
    p.add_argument("--rounds", type=int, default=10, help="repeats per settle value")
    p.add_argument("--ensure-timeout-ms", type=int, default=500)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    settle_list = [max(0, int(x.strip())) for x in args.settle_list_ms.split(",") if x.strip()]
    rounds = max(1, int(args.rounds))

    print(
        f"sop=05-setzero-timing-ab serial={args.serial_port}@{args.serial_baud} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} rounds={rounds} settle_list={settle_list}"
    )

    for settle_ms in settle_list:
        ok, fail = run_one_case(
            serial_port=args.serial_port,
            serial_baud=int(args.serial_baud),
            model=args.model,
            motor_id=motor_id,
            feedback_id=feedback_id,
            settle_ms=settle_ms,
            rounds=rounds,
            ensure_timeout_ms=int(args.ensure_timeout_ms),
        )
        total = ok + fail
        print(
            f"settle_ms={settle_ms} total={total} ok={ok} fail={fail} "
            f"success_rate={(ok / total) if total else 0.0:.4f}"
        )


if __name__ == "__main__":
    main()

