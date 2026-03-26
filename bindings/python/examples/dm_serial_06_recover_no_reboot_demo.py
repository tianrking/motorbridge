#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, RID_CTRL_MODE


def _parse_id(text: str) -> int:
    return int(text, 0)


def _recover_once(
    serial_port: str,
    serial_baud: int,
    model: str,
    motor_id: int,
    feedback_id: int,
    timeout_ms: int,
    probe_rounds: int,
    probe_interval_ms: int,
) -> tuple[bool, str]:
    with Controller.from_dm_serial(serial_port, serial_baud) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, model)
        try:
            # Soft reset sequence without rebooting host.
            try:
                m.disable()
            except Exception:
                pass
            time.sleep(0.05)

            try:
                m.clear_error()
            except Exception:
                pass

            try:
                ctrl.enable_all()
            except Exception:
                pass
            time.sleep(0.1)

            # Probe register path multiple times; this is what usually breaks.
            for _ in range(max(1, probe_rounds)):
                try:
                    mode_raw = m.get_register_u32(RID_CTRL_MODE, timeout_ms)
                    return True, f"RID_CTRL_MODE={mode_raw}"
                except Exception as e:
                    last_err = str(e)
                    try:
                        m.request_feedback()
                        ctrl.poll_feedback_once()
                    except Exception:
                        pass
                    time.sleep(max(0, probe_interval_ms) / 1000.0)
            return False, f"register path not recovered: {last_err}"
        finally:
            try:
                m.disable()
            except Exception:
                pass
            m.close()


def main() -> None:
    p = argparse.ArgumentParser(
        description="SOP-06 (dm-serial): recover register-read path without host reboot"
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x04")
    p.add_argument("--feedback-id", default="0x14")
    p.add_argument("--attempts", type=int, default=6)
    p.add_argument("--timeout-ms", type=int, default=800, help="RID read timeout")
    p.add_argument("--probe-rounds", type=int, default=3, help="register probes per attempt")
    p.add_argument("--probe-interval-ms", type=int, default=50)
    p.add_argument("--backoff-ms", type=int, default=200)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    attempts = max(1, int(args.attempts))

    print(
        f"sop=06-recover-no-reboot serial={args.serial_port}@{args.serial_baud} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} attempts={attempts}"
    )

    for i in range(attempts):
        ok, msg = _recover_once(
            serial_port=args.serial_port,
            serial_baud=int(args.serial_baud),
            model=args.model,
            motor_id=motor_id,
            feedback_id=feedback_id,
            timeout_ms=max(1, int(args.timeout_ms)),
            probe_rounds=max(1, int(args.probe_rounds)),
            probe_interval_ms=max(0, int(args.probe_interval_ms)),
        )
        if ok:
            print(f"attempt={i+1}/{attempts} result=ok {msg}")
            return
        print(f"attempt={i+1}/{attempts} result=fail {msg}")
        time.sleep(max(0, int(args.backoff_ms)) / 1000.0)

    raise SystemExit("recovery failed after all attempts")


if __name__ == "__main__":
    main()

