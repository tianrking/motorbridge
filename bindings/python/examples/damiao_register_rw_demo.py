#!/usr/bin/env python3
from __future__ import annotations

import argparse

from motorbridge import Controller


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(
        description="Damiao register read/write demo (f32 + u32 + optional store)"
    )
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4340P")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")
    p.add_argument("--timeout-ms", type=int, default=1000)

    p.add_argument("--read-f32-rid", default="21", help="default PMAX")
    p.add_argument("--read-u32-rid", default="10", help="default CTRL_MODE")

    p.add_argument("--write-f32-rid", default="", help="optional rid")
    p.add_argument("--write-f32-value", type=float, default=0.0)
    p.add_argument("--write-u32-rid", default="", help="optional rid")
    p.add_argument("--write-u32-value", default="0")
    p.add_argument("--store", type=int, default=0, help="1 to call store_parameters")
    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    read_f32_rid = _parse_id(args.read_f32_rid)
    read_u32_rid = _parse_id(args.read_u32_rid)

    print(
        f"channel={args.channel} model={args.model} motor_id=0x{motor_id:X} "
        f"feedback_id=0x{feedback_id:X}"
    )

    with Controller(args.channel) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            f32_v = m.get_register_f32(read_f32_rid, args.timeout_ms)
            u32_v = m.get_register_u32(read_u32_rid, args.timeout_ms)
            print(f"read_f32 rid={read_f32_rid} value={f32_v}")
            print(f"read_u32 rid={read_u32_rid} value={u32_v}")

            if args.write_f32_rid:
                rid = _parse_id(args.write_f32_rid)
                m.write_register_f32(rid, args.write_f32_value)
                back = m.get_register_f32(rid, args.timeout_ms)
                print(f"write_f32 rid={rid} value={args.write_f32_value} verify={back}")

            if args.write_u32_rid:
                rid = _parse_id(args.write_u32_rid)
                value = _parse_id(args.write_u32_value)
                m.write_register_u32(rid, value)
                back = m.get_register_u32(rid, args.timeout_ms)
                print(f"write_u32 rid={rid} value={value} verify={back}")

            if args.store:
                m.store_parameters()
                print("store_parameters done")
        finally:
            m.close()


if __name__ == "__main__":
    main()

