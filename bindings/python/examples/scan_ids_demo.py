#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, RID_CTRL_MODE


def _parse_id(text: str) -> int:
    return int(text, 0)


def _hx(v: int) -> str:
    return f"0x{v:03X}"


def main() -> None:
    p = argparse.ArgumentParser(
        description="Fast CAN-bus scan demo (register probe + enable fallback)"
    )
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4310")
    p.add_argument("--start-id", default="0x001")
    p.add_argument("--end-id", default="0x7FF")
    p.add_argument("--feedback-base", default="0x10")
    p.add_argument("--timeout-ms", type=int, default=80)
    p.add_argument("--sleep-ms", type=int, default=2)
    p.add_argument("--verbose", type=int, default=0)
    args = p.parse_args()

    start_id = _parse_id(args.start_id)
    end_id = _parse_id(args.end_id)
    feedback_base = _parse_id(args.feedback_base)

    if start_id < 0x001 or end_id > 0x7FF or start_id > end_id:
        raise SystemExit("invalid range: use 0x001..0x7FF and start <= end")

    print(
        f"scan start: channel={args.channel} model={args.model} "
        f"range={_hx(start_id)}..{_hx(end_id)} feedback-base={_hx(feedback_base)} "
        f"timeout-ms={args.timeout_ms}"
    )

    hits = 0
    with Controller(args.channel) as ctrl:
        for mid in range(start_id, end_id + 1):
            fid = feedback_base + (mid & 0x0F)
            ok = False
            try:
                m = ctrl.add_damiao_motor(mid, fid, args.model)
                try:
                    _ = m.get_register_u32(RID_CTRL_MODE, args.timeout_ms)
                    ok = True
                except Exception as e_reg:
                    if args.verbose:
                        print(
                            f"[probe-reg-miss] motor-id={_hx(mid)} feedback-id={_hx(fid)} err={e_reg}"
                        )
                    try:
                        m.enable()
                        m.request_feedback()
                        for _ in range(3):
                            ctrl.poll_feedback_once()
                            if m.get_state() is not None:
                                ok = True
                                break
                            time.sleep(0.005)
                    except Exception as e_en:
                        if args.verbose:
                            print(
                                f"[probe-enable-miss] motor-id={_hx(mid)} feedback-id={_hx(fid)} err={e_en}"
                            )
                finally:
                    m.close()
            except Exception as e_add:
                if args.verbose:
                    print(f"[add-motor-miss] motor-id={_hx(mid)} feedback-id={_hx(fid)} err={e_add}")

            if ok:
                print(f"[hit] motor-id={_hx(mid)} feedback-id={_hx(fid)}", flush=True)
                hits += 1

            if args.sleep_ms > 0:
                time.sleep(args.sleep_ms / 1000.0)

    print(f"scan done, hits={hits}")


if __name__ == "__main__":
    main()

