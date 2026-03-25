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


def _align_list(values: list[float], size: int, name: str, default: float | None = None) -> list[float]:
    if not values:
        if default is None:
            raise ValueError(f"{name} is required")
        return [default] * size
    if len(values) == 1:
        return values * size
    if len(values) == size:
        return values
    raise ValueError(
        f"{name} count mismatch: got {len(values)}, expected 1 or {size} "
        f"(same order as IDs)"
    )


def _mode_to_enum(mode: str) -> Mode:
    return {
        "mit": Mode.MIT,
        "pos-vel": Mode.POS_VEL,
        "vel": Mode.VEL,
        "force-pos": Mode.FORCE_POS,
    }[mode]


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


def _now_ms() -> float:
    return time.perf_counter() * 1000.0


def _summary(name: str, values_ms: list[float]) -> str:
    if not values_ms:
        return f"{name}: no samples"
    total = sum(values_ms)
    return (
        f"{name}: count={len(values_ms)} avg={total / len(values_ms):.3f}ms "
        f"min={min(values_ms):.3f}ms max={max(values_ms):.3f}ms total={total:.3f}ms"
    )


def _trace_sdk_call(
    enabled: bool,
    name: str,
    fn,
    *args,
    **kwargs,
):
    t0 = _now_ms()
    out = fn(*args, **kwargs)
    dt = _now_ms() - t0
    if enabled:
        print(f"[sdk] {name} took {dt:.3f} ms")
    return out, dt


def main() -> None:
    p = argparse.ArgumentParser(description="Multi-motor control demo with one-to-one argument mapping")
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4310")
    p.add_argument(
        "--mode",
        default="pos-vel",
        choices=["mit", "pos-vel", "vel", "force-pos"],
    )

    # Keep short options close to your requested style: -id / -pos / -vel.
    p.add_argument("-id", "--ids", nargs="+", required=True, help="motor ids, e.g. -id 4 7 or -id 0x04,0x07")
    p.add_argument(
        "-fid",
        "--feedback-id",
        nargs="+",
        default=[],
        help="optional feedback ids; if omitted uses feedback-base + (id & 0x0F)",
    )
    p.add_argument("--feedback-base", default="0x10")

    p.add_argument("-pos", "--pos", nargs="+", default=[], help="position list in rad")
    p.add_argument("-vlim", "--vlim", nargs="+", default=["1.5"], help="velocity limit list")
    p.add_argument("-vel", "--vel", nargs="+", default=[], help="velocity list")
    p.add_argument("-kp", "--kp", nargs="+", default=["30.0"], help="mit kp list")
    p.add_argument("-kd", "--kd", nargs="+", default=["1.0"], help="mit kd list")
    p.add_argument("-tau", "--tau", nargs="+", default=["0.0"], help="mit torque list")
    p.add_argument("-ratio", "--ratio", nargs="+", default=["0.3"], help="force-pos ratio list")

    p.add_argument("--loop", type=int, default=200)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--ensure-timeout-ms", type=int, default=1000)
    p.add_argument("--print-state", type=int, default=1)
    p.add_argument("--timing-log", type=int, default=1, help="1 to print timing logs")
    p.add_argument("--timing-every", type=int, default=1, help="print per-loop timing every N loops")
    p.add_argument("--trace-sdk", type=int, default=1, help="1 to print every SDK call timing")
    args = p.parse_args()

    motor_ids = _parse_id_list(args.ids)
    if not motor_ids:
        raise ValueError("at least one motor id is required")

    feedback_ids = _feedback_ids_for(
        motor_ids,
        _parse_id_list(args.feedback_id),
        int(args.feedback_base, 0),
    )

    pos = _align_list(_parse_float_list(args.pos), len(motor_ids), "pos", default=None if args.mode in ("pos-vel", "mit", "force-pos") else 0.0)
    vlim = _align_list(_parse_float_list(args.vlim), len(motor_ids), "vlim", default=1.5)
    vel = _align_list(_parse_float_list(args.vel), len(motor_ids), "vel", default=0.0)
    kp = _align_list(_parse_float_list(args.kp), len(motor_ids), "kp", default=30.0)
    kd = _align_list(_parse_float_list(args.kd), len(motor_ids), "kd", default=1.0)
    tau = _align_list(_parse_float_list(args.tau), len(motor_ids), "tau", default=0.0)
    ratio = _align_list(_parse_float_list(args.ratio), len(motor_ids), "ratio", default=0.3)

    print(
        f"channel={args.channel} model={args.model} mode={args.mode} ids={[hex(x) for x in motor_ids]} "
        f"feedback_ids={[hex(x) for x in feedback_ids]}"
    )

    motors = []
    add_motor_ms: list[float] = []
    enable_all_ms: list[float] = []
    ensure_mode_ms: list[float] = []
    ensure_mode_by_motor: dict[int, float] = {}
    send_loop_ms: list[float] = []
    send_by_motor: dict[int, list[float]] = {mid: [] for mid in motor_ids}
    get_state_ms: list[float] = []

    with Controller(args.channel) as ctrl:
        try:
            for mid, fid in zip(motor_ids, feedback_ids):
                m, dt = _trace_sdk_call(
                    bool(args.trace_sdk),
                    f"add_damiao_motor motor=0x{mid:X} feedback=0x{fid:X}",
                    ctrl.add_damiao_motor,
                    mid,
                    fid,
                    args.model,
                )
                add_motor_ms.append(dt)
                if args.timing_log:
                    print(f"[timing] add_damiao_motor motor=0x{mid:X} feedback=0x{fid:X} took {dt:.3f} ms")
                motors.append(m)

            _, dt = _trace_sdk_call(bool(args.trace_sdk), "enable_all", ctrl.enable_all)
            enable_all_ms.append(dt)
            if args.timing_log:
                print(f"[timing] enable_all took {dt:.3f} ms")
            time.sleep(0.3)

            mode_enum = _mode_to_enum(args.mode)
            for idx, m in enumerate(motors):
                _, dt = _trace_sdk_call(
                    bool(args.trace_sdk),
                    f"ensure_mode motor=0x{motor_ids[idx]:X} mode={args.mode}",
                    m.ensure_mode,
                    mode_enum,
                    args.ensure_timeout_ms,
                )
                ensure_mode_ms.append(dt)
                ensure_mode_by_motor[motor_ids[idx]] = dt
                if args.timing_log:
                    print(f"[timing] ensure_mode motor=0x{motor_ids[idx]:X} mode={args.mode} took {dt:.3f} ms")

            for i in range(args.loop):
                loop_t0 = _now_ms()
                for idx, m in enumerate(motors):
                    if args.mode == "pos-vel":
                        _, cmd_dt = _trace_sdk_call(
                            bool(args.trace_sdk),
                            f"send_pos_vel loop={i} motor=0x{motor_ids[idx]:X}",
                            m.send_pos_vel,
                            pos[idx],
                            vlim[idx],
                        )
                    elif args.mode == "vel":
                        _, cmd_dt = _trace_sdk_call(
                            bool(args.trace_sdk),
                            f"send_vel loop={i} motor=0x{motor_ids[idx]:X}",
                            m.send_vel,
                            vel[idx],
                        )
                    elif args.mode == "mit":
                        _, cmd_dt = _trace_sdk_call(
                            bool(args.trace_sdk),
                            f"send_mit loop={i} motor=0x{motor_ids[idx]:X}",
                            m.send_mit,
                            pos[idx],
                            vel[idx],
                            kp[idx],
                            kd[idx],
                            tau[idx],
                        )
                    elif args.mode == "force-pos":
                        _, cmd_dt = _trace_sdk_call(
                            bool(args.trace_sdk),
                            f"send_force_pos loop={i} motor=0x{motor_ids[idx]:X}",
                            m.send_force_pos,
                            pos[idx],
                            vlim[idx],
                            ratio[idx],
                        )
                    send_by_motor[motor_ids[idx]].append(cmd_dt)

                if args.print_state:
                    for idx, m in enumerate(motors):
                        st, st_dt = _trace_sdk_call(
                            bool(args.trace_sdk),
                            f"get_state loop={i} motor=0x{motor_ids[idx]:X}",
                            m.get_state,
                        )
                        get_state_ms.append(st_dt)
                        if st is None:
                            print(f"#{i} motor=0x{motor_ids[idx]:X} no feedback yet")
                        else:
                            print(
                                f"#{i} motor=0x{motor_ids[idx]:X} "
                                f"pos={st.pos:+.3f} vel={st.vel:+.3f} torq={st.torq:+.3f} status={st.status_code}"
                            )

                loop_dt = _now_ms() - loop_t0
                send_loop_ms.append(loop_dt)
                if args.timing_log and args.timing_every > 0 and (i % args.timing_every == 0):
                    per_motor = " ".join(
                        f"0x{mid:X}:{send_by_motor[mid][-1]:.3f}ms" for mid in motor_ids if send_by_motor[mid]
                    )
                    print(f"[timing] loop={i} send+state={loop_dt:.3f} ms per_send=[{per_motor}]")

                if args.dt_ms > 0:
                    time.sleep(args.dt_ms / 1000.0)
        finally:
            if args.timing_log:
                print("[timing] ===== summary =====")
                print(f"[timing] {_summary('add_damiao_motor', add_motor_ms)}")
                print(f"[timing] {_summary('enable_all', enable_all_ms)}")
                print(f"[timing] {_summary('ensure_mode', ensure_mode_ms)}")
                for mid in motor_ids:
                    one = [ensure_mode_by_motor[mid]] if mid in ensure_mode_by_motor else []
                    print(f"[timing] {_summary(f'ensure_mode motor=0x{mid:X}', one)}")
                print(f"[timing] {_summary('send loop (send+state)', send_loop_ms)}")
                for mid in motor_ids:
                    print(f"[timing] {_summary(f'send command motor=0x{mid:X}', send_by_motor[mid])}")
                if args.print_state:
                    print(f"[timing] {_summary('get_state', get_state_ms)}")
            for m in motors:
                try:
                    _trace_sdk_call(bool(args.trace_sdk), "motor.close", m.close)
                except Exception:
                    pass


if __name__ == "__main__":
    main()
