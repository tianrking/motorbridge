#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def _run_rotate_phase(
    ctrl: Controller,
    motor,
    phase_name: str,
    target_pos: float,
    vlim: float,
    loop: int,
    dt_ms: int,
    ensure_timeout_ms: int,
) -> None:
    print(f"[{phase_name}] ensure_mode -> POS_VEL")
    motor.ensure_mode(Mode.POS_VEL, ensure_timeout_ms)
    dt_s = max(0, int(dt_ms)) / 1000.0
    for i in range(max(1, int(loop))):
        motor.send_pos_vel(target_pos, vlim)
        try:
            motor.request_feedback()
            ctrl.poll_feedback_once()
        except Exception:
            pass
        st = motor.get_state()
        if st is None:
            print(f"[{phase_name}] #{i+1}/{loop} no_feedback")
        else:
            print(
                f"[{phase_name}] #{i+1}/{loop} pos={st.pos:+.3f} vel={st.vel:+.3f} "
                f"torq={st.torq:+.3f} status={st.status_code}"
            )
        if dt_s > 0:
            time.sleep(dt_s)


def main() -> None:
    p = argparse.ArgumentParser(
        description=(
            "SOP-07 (dm-serial): disable -> set_zero -> enable -> rotate(3rad), "
            "then disable -> set_zero -> enable -> rotate again"
        )
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x04")
    p.add_argument("--feedback-id", default="0x14")
    p.add_argument("--target-pos", type=float, default=3.0)
    p.add_argument("--vlim", type=float, default=1.0)
    p.add_argument("--loop", type=int, default=50)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--ensure-timeout-ms", type=int, default=800)
    p.add_argument("--settle-ms", type=int, default=200, help="wait after set_zero_position")
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    settle_s = max(0, int(args.settle_ms)) / 1000.0

    print(
        f"sop=07-enable-setzero-enable-rotate serial={args.serial_port}@{args.serial_baud} "
        f"model={args.model} motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} "
        f"target_pos={args.target_pos} vlim={args.vlim} settle_ms={args.settle_ms}"
    )

    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            for phase_idx in (1, 2):
                phase = f"phase-{phase_idx}"
                print(f"[{phase}] disable before set_zero")
                try:
                    m.disable()
                except Exception:
                    pass
                time.sleep(0.1)

                print(f"[{phase}] set_zero_position")
                m.set_zero_position()
                if settle_s > 0:
                    time.sleep(settle_s)

                print(f"[{phase}] enable_all (again)")
                ctrl.enable_all()

                _run_rotate_phase(
                    ctrl=ctrl,
                    motor=m,
                    phase_name=phase,
                    target_pos=float(args.target_pos),
                    vlim=float(args.vlim),
                    loop=int(args.loop),
                    dt_ms=int(args.dt_ms),
                    ensure_timeout_ms=int(args.ensure_timeout_ms),
                )
        finally:
            try:
                m.disable()
            except Exception:
                pass
            m.close()


if __name__ == "__main__":
    main()
