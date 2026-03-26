#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, RID_CTRL_MODE


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(
        description=(
            "Damiao dm-serial BER-like probe: send fixed-count register query commands "
            "at target frequency and report failure rate"
        )
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--motor-id", default="0x04")
    p.add_argument("--feedback-id", default="0x14")
    p.add_argument(
        "--freq-hz",
        type=float,
        default=0.0,
        help="target query frequency (Hz). <=0 means use --interval-ms directly",
    )
    p.add_argument(
        "--interval-ms",
        type=float,
        default=5.0,
        help="query interval in ms (used when --freq-hz <= 0)",
    )
    p.add_argument(
        "--packets",
        type=int,
        default=200,
        help="number of query commands to send (default 200)",
    )
    p.add_argument("--rounds", type=int, default=1, help="repeat rounds per interval")
    p.add_argument(
        "--timeout-ms",
        type=int,
        default=80,
        help="per-query timeout for register response",
    )
    p.add_argument(
        "--enable-all",
        type=int,
        default=1,
        help="1 to call ctrl.enable_all() before probing",
    )
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    packets = max(1, int(args.packets))
    if float(args.freq_hz) > 0:
        period_s = 1.0 / float(args.freq_hz)
    else:
        period_s = max(0.0005, float(args.interval_ms) / 1000.0)
    target_hz = 1.0 / period_s
    timeout_ms = max(1, int(args.timeout_ms))
    rounds = max(1, int(args.rounds))

    tx_total = 0
    rx_ok = 0
    rx_fail = 0
    overtime_count = 0
    mode_last: int | None = None

    print(
        f"probe=dm-serial-query-ber serial={args.serial_port}@{args.serial_baud} "
        f"model={args.model} motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} "
        f"freq_hz={target_hz:.3f} interval_ms={period_s * 1000.0:.3f} "
        f"packets={packets} rounds={rounds} timeout_ms={timeout_ms}"
    )

    t_start = time.perf_counter()
    with Controller.from_dm_serial(args.serial_port, args.serial_baud) as ctrl:
        motor = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            if args.enable_all:
                ctrl.enable_all()
                time.sleep(0.2)

            for round_idx in range(rounds):
                round_ok = 0
                round_fail = 0
                round_over = 0
                next_tick = time.perf_counter()
                for _ in range(packets):
                    tx_total += 1
                    t0 = time.perf_counter()
                    try:
                        mode_last = motor.get_register_u32(RID_CTRL_MODE, timeout_ms)
                        rx_ok += 1
                        round_ok += 1
                    except Exception:
                        rx_fail += 1
                        round_fail += 1

                    elapsed = time.perf_counter() - t0
                    if elapsed > period_s:
                        overtime_count += 1
                        round_over += 1

                    next_tick += period_s
                    sleep_s = next_tick - time.perf_counter()
                    if sleep_s > 0:
                        time.sleep(sleep_s)
                print(
                    f"round={round_idx + 1}/{rounds} tx={packets} ok={round_ok} fail={round_fail} "
                    f"success_rate={round_ok / packets:.4f} ber={round_fail / packets:.4f} "
                    f"overtime={round_over}"
                )
        finally:
            motor.close()

    total_s = max(1e-9, time.perf_counter() - t_start)
    achieved_hz = tx_total / total_s if tx_total > 0 else 0.0
    fail_rate = rx_fail / tx_total if tx_total > 0 else 0.0
    success_rate = rx_ok / tx_total if tx_total > 0 else 0.0

    print(
        f"result tx_total={tx_total} rx_ok={rx_ok} rx_fail={rx_fail} "
        f"success_rate={success_rate:.4f} ber={fail_rate:.4f} "
        f"overtime={overtime_count} achieved_hz={achieved_hz:.2f}"
    )
    if mode_last is not None:
        print(f"last_ctrl_mode={mode_last}")


if __name__ == "__main__":
    main()
