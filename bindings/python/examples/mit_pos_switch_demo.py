#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


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


def _parse_float_list(tokens: list[str]) -> list[float]:
    return [float(x) for x in _flatten_tokens(tokens)]


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


def _print_states(prefix: str, motor_ids: list[int], motors, enabled: bool) -> None:
    if not enabled:
        return
    for idx, m in enumerate(motors):
        st = m.get_state()
        if st is None:
            print(f"{prefix} motor=0x{motor_ids[idx]:X} no feedback yet")
        else:
            print(
                f"{prefix} motor=0x{motor_ids[idx]:X} "
                f"pos={st.pos:+.3f} vel={st.vel:+.3f} torq={st.torq:+.3f} status={st.status_code}"
            )


def _verify_ctrl_mode(motor_ids: list[int], motors, expected: int, timeout_ms: int) -> None:
    for idx, m in enumerate(motors):
        actual = m.get_register_u32(10, timeout_ms)
        ok = "OK" if actual == expected else "MISMATCH"
        print(
            f"[ensure-check] motor=0x{motor_ids[idx]:X} CTRL_MODE={actual} "
            f"expected={expected} {ok}"
        )


def main() -> None:
    p = argparse.ArgumentParser(
        description="Two-phase Damiao demo: MIT trajectory once, then POS_VEL trajectory once"
    )
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4310")

    p.add_argument("-id", "--ids", nargs="+", default=["4", "7"], help="motor ids, e.g. -id 4 7")
    p.add_argument("-fid", "--feedback-id", nargs="+", default=[], help="optional feedback ids")
    p.add_argument("--feedback-base", default="0x10")

    p.add_argument(
        "--trajectory",
        nargs="+",
        default=["0", "-3", "0", "-3"],
        help="position sequence (rad), e.g. --trajectory 0 -3 0 -3",
    )

    p.add_argument("--ensure-timeout-ms", type=int, default=1000)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--print-state", type=int, default=1)

    # MIT phase params
    p.add_argument("--mit-vel", type=float, default=0.0)
    p.add_argument("--mit-kp", type=float, default=30.0)
    p.add_argument("--mit-kd", type=float, default=1.0)
    p.add_argument("--mit-tau", type=float, default=0.0)
    p.add_argument("--mit-hold-loops", type=int, default=50, help="loops per trajectory point")

    # POS_VEL phase params
    p.add_argument("--pos-vlim", type=float, default=1.5)
    p.add_argument("--pos-hold-loops", type=int, default=50, help="loops per trajectory point")

    args = p.parse_args()

    motor_ids = _parse_id_list(args.ids)
    if len(motor_ids) < 1:
        raise ValueError("at least one motor id is required")

    feedback_ids = _feedback_ids_for(
        motor_ids,
        _parse_id_list(args.feedback_id),
        int(args.feedback_base, 0),
    )
    trajectory = _parse_float_list(args.trajectory)
    if len(trajectory) < 1:
        raise ValueError("trajectory must contain at least one target")

    print(
        f"channel={args.channel} model={args.model} ids={[hex(x) for x in motor_ids]} "
        f"feedback_ids={[hex(x) for x in feedback_ids]} trajectory={trajectory}"
    )

    motors = []
    with Controller(args.channel) as ctrl:
        try:
            for mid, fid in zip(motor_ids, feedback_ids):
                motors.append(ctrl.add_damiao_motor(mid, fid, args.model))

            ctrl.enable_all()
            time.sleep(0.3)

            # Phase 1: MIT
            print("[phase] MIT")
            for m in motors:
                m.ensure_mode(Mode.MIT, args.ensure_timeout_ms)
            # _verify_ctrl_mode(motor_ids, motors, expected=1, timeout_ms=args.ensure_timeout_ms)

            for point_idx, target in enumerate(trajectory):
                print(f"[mit] point#{point_idx} target={target}")
                for loop_idx in range(args.mit_hold_loops):
                    for m in motors:
                        m.send_mit(target, args.mit_vel, args.mit_kp, args.mit_kd, args.mit_tau)
                    _print_states(
                        prefix=f"[mit#{point_idx}:{loop_idx}]",
                        motor_ids=motor_ids,
                        motors=motors,
                        enabled=bool(args.print_state),
                    )
                    if args.dt_ms > 0:
                        time.sleep(args.dt_ms / 1000.0)

            # Phase 2: POS_VEL
            print("[phase] POS_VEL")
            for m in motors:
                m.ensure_mode(Mode.POS_VEL, args.ensure_timeout_ms)
            # _verify_ctrl_mode(motor_ids, motors, expected=2, timeout_ms=args.ensure_timeout_ms)

            for point_idx, target in enumerate(trajectory):
                print(f"[pos] point#{point_idx} target={target}")
                for loop_idx in range(args.pos_hold_loops):
                    for m in motors:
                        m.send_pos_vel(target, args.pos_vlim)
                    _print_states(
                        prefix=f"[pos#{point_idx}:{loop_idx}]",
                        motor_ids=motor_ids,
                        motors=motors,
                        enabled=bool(args.print_state),
                    )
                    if args.dt_ms > 0:
                        time.sleep(args.dt_ms / 1000.0)

            print("[done] MIT + POS_VEL trajectory completed")
        finally:
            for m in motors:
                try:
                    m.close()
                except Exception:
                    pass


if __name__ == "__main__":
    main()
