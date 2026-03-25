#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import (
    Controller,
    RID_CTRL_MODE,
    RID_ESC_ID,
    RID_MST_ID,
    RID_PMAX,
    RID_TMAX,
    RID_TIMEOUT,
    RID_VMAX,
)


def _parse_id(text: str) -> int:
    return int(text, 0)


def _mode_name(v: int) -> str:
    return {
        1: "MIT",
        2: "POS_VEL",
        3: "VEL",
        4: "FORCE_POS",
    }.get(v, f"UNKNOWN({v})")


def _safe_read_u32(motor, rid: int, timeout_ms: int) -> str:
    try:
        return str(motor.get_register_u32(rid, timeout_ms))
    except Exception as e:
        return f"ERR({e})"


def _safe_read_f32(motor, rid: int, timeout_ms: int) -> str:
    try:
        return f"{motor.get_register_f32(rid, timeout_ms):.3f}"
    except Exception as e:
        return f"ERR({e})"


def main() -> None:
    p = argparse.ArgumentParser(
        description="dm-serial status demo: print current mode + key registers + feedback state"
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x07")
    p.add_argument("--feedback-id", default="0x17")
    p.add_argument("--timeout-ms", type=int, default=300)
    p.add_argument("--loop", type=int, default=50)
    p.add_argument("--dt-ms", type=int, default=100)
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    timeout_ms = max(1, int(args.timeout_ms))
    loop = max(1, int(args.loop))
    dt_s = max(0, int(args.dt_ms)) / 1000.0

    print(
        f"demo=dm-serial-status serial={args.serial_port}@{args.serial_baud} "
        f"model={args.model} motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X}"
    )

    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        motor = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            for i in range(loop):
                # Active feedback refresh (same idea as CLI loop).
                try:
                    motor.request_feedback()
                    ctrl.poll_feedback_once()
                except Exception:
                    pass

                st = motor.get_state()
                mode_raw = _safe_read_u32(motor, RID_CTRL_MODE, timeout_ms)
                mode_desc = _mode_name(int(mode_raw)) if mode_raw.isdigit() else mode_raw
                esc_id = _safe_read_u32(motor, RID_ESC_ID, timeout_ms)
                mst_id = _safe_read_u32(motor, RID_MST_ID, timeout_ms)
                can_timeout = _safe_read_u32(motor, RID_TIMEOUT, timeout_ms)
                pmax = _safe_read_f32(motor, RID_PMAX, timeout_ms)
                vmax = _safe_read_f32(motor, RID_VMAX, timeout_ms)
                tmax = _safe_read_f32(motor, RID_TMAX, timeout_ms)

                if st is None:
                    state_text = "state=no_feedback"
                else:
                    state_text = (
                        f"state=ok pos={st.pos:+.3f} vel={st.vel:+.3f} torq={st.torq:+.3f} "
                        f"status={st.status_code} t_mos={st.t_mos:.1f}C t_rotor={st.t_rotor:.1f}C"
                    )

                print(
                    f"#{i+1}/{loop} mode={mode_desc} ctrl_mode_raw={mode_raw} "
                    f"esc_id={esc_id} mst_id={mst_id} can_timeout={can_timeout} "
                    f"limits(pmax,vmax,tmax)=({pmax},{vmax},{tmax}) {state_text}"
                )

                if dt_s > 0:
                    time.sleep(dt_s)
        finally:
            motor.close()


if __name__ == "__main__":
    main()

