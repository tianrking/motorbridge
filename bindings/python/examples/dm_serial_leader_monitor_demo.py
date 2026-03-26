#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode, RID_CTRL_MODE


def _flatten_tokens(tokens: list[str]) -> list[str]:
    out: list[str] = []
    for token in tokens:
        for item in token.split(","):
            item = item.strip()
            if item:
                out.append(item)
    return out


def _parse_id_list(tokens: list[str]) -> list[int]:
    return [int(x, 0) for x in _flatten_tokens(tokens)]


def _feedback_ids_for(motor_ids: list[int], feedback_ids: list[int], feedback_base: int) -> list[int]:
    if not feedback_ids:
        return [feedback_base + (mid & 0x0F) for mid in motor_ids]
    if len(feedback_ids) == 1:
        return feedback_ids * len(motor_ids)
    if len(feedback_ids) == len(motor_ids):
        return feedback_ids
    raise ValueError(
        f"feedback-id count mismatch: got {len(feedback_ids)}, expected 1 or {len(motor_ids)}"
    )


def _mode_name(raw: int) -> str:
    return {
        1: "MIT",
        2: "POS_VEL",
        3: "VEL",
        4: "FORCE_POS",
    }.get(raw, f"UNKNOWN({raw})")


def main() -> None:
    p = argparse.ArgumentParser(
        description="Damiao dm-serial leader monitor: enable all, then stream full state for selected IDs"
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")

    p.add_argument("-id", "--ids", nargs="+", required=True, help="motor ids, e.g. -id 4 7 or -id 0x04,0x07")
    p.add_argument("-fid", "--feedback-id", nargs="+", default=[], help="optional feedback ids")
    p.add_argument("--feedback-base", default="0x10")

    p.add_argument("--loop", type=int, default=10000)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--ensure-timeout-ms", type=int, default=300)
    p.add_argument("--read-mode-every", type=int, default=10, help="read CTRL_MODE every N loops")

    # "low-impedance feel" hold mode: keep enabled while injecting near-zero MIT command.
    p.add_argument(
        "--hold-mode",
        choices=["none", "mit-zero"],
        default="mit-zero",
        help="none: pure read; mit-zero: keep enabled with near-zero MIT command",
    )
    p.add_argument("--mit-pos", type=float, default=0.0)
    p.add_argument("--mit-vel", type=float, default=0.0)
    p.add_argument("--mit-kp", type=float, default=0.0)
    p.add_argument("--mit-kd", type=float, default=0.01)
    p.add_argument("--mit-tau", type=float, default=0.0)

    p.add_argument("--disable-on-exit", type=int, default=1)
    args = p.parse_args()

    motor_ids = _parse_id_list(args.ids)
    if not motor_ids:
        raise ValueError("at least one motor id is required")
    feedback_ids = _feedback_ids_for(
        motor_ids,
        _parse_id_list(args.feedback_id),
        int(args.feedback_base, 0),
    )

    loop_count = max(1, int(args.loop))
    dt_s = max(0, int(args.dt_ms)) / 1000.0
    read_mode_every = max(1, int(args.read_mode_every))

    print(
        f"demo=dm-serial-leader-monitor serial={args.serial_port}@{args.serial_baud} model={args.model} "
        f"ids={[hex(x) for x in motor_ids]} feedback_ids={[hex(x) for x in feedback_ids]} "
        f"hold_mode={args.hold_mode} loop={loop_count} dt_ms={args.dt_ms}"
    )

    motors = []
    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        try:
            for mid, fid in zip(motor_ids, feedback_ids):
                motors.append(ctrl.add_damiao_motor(mid, fid, args.model))

            # Enable all selected motors once at startup.
            ctrl.enable_all()
            time.sleep(0.2)

            # Optional low-impedance hold: ensure MIT once, then send near-zero commands each loop.
            if args.hold_mode == "mit-zero":
                for m in motors:
                    m.ensure_mode(Mode.MIT, args.ensure_timeout_ms)

            for i in range(loop_count):
                if args.hold_mode == "mit-zero":
                    for m in motors:
                        m.send_mit(
                            args.mit_pos,
                            args.mit_vel,
                            args.mit_kp,
                            args.mit_kd,
                            args.mit_tau,
                        )

                # Ask all motors for fresh feedback.
                for m in motors:
                    try:
                        m.request_feedback()
                    except Exception:
                        pass

                # Poll multiple frames to increase chance of collecting all motor updates.
                for _ in range(len(motors) * 2):
                    try:
                        ctrl.poll_feedback_once()
                    except Exception:
                        break

                # Print full state for each selected motor.
                for idx, m in enumerate(motors):
                    mode_text = "-"
                    if i % read_mode_every == 0:
                        try:
                            mode_raw = m.get_register_u32(RID_CTRL_MODE, args.ensure_timeout_ms)
                            mode_text = f"{_mode_name(mode_raw)}({mode_raw})"
                        except Exception as e:
                            mode_text = f"ERR({e})"

                    st = m.get_state()
                    if st is None:
                        print(
                            f"#{i+1}/{loop_count} motor=0x{motor_ids[idx]:X} "
                            f"mode={mode_text} no_feedback"
                        )
                    else:
                        print(
                            f"#{i+1}/{loop_count} motor=0x{motor_ids[idx]:X} "
                            f"mode={mode_text} "
                            f"pos={st.pos:+.3f} vel={st.vel:+.3f} torq={st.torq:+.3f} "
                            f"status={st.status_code} can_id={st.can_id} arb=0x{st.arbitration_id:03X} "
                            f"t_mos={st.t_mos:.1f}C t_rotor={st.t_rotor:.1f}C"
                        )

                if dt_s > 0:
                    time.sleep(dt_s)
        finally:
            if args.disable_on_exit:
                try:
                    ctrl.disable_all()
                except Exception:
                    pass
            for m in motors:
                try:
                    m.close()
                except Exception:
                    pass


if __name__ == "__main__":
    main()

