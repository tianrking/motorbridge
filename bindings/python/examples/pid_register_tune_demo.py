#!/usr/bin/env python3
from __future__ import annotations

import argparse

from motorbridge import Controller

RID_ACC = 4
RID_DEC = 5
RID_MAX_SPD = 6
RID_PMAX = 21
RID_VMAX = 22
RID_TMAX = 23
RID_KP_ASR = 25
RID_KI_ASR = 26
RID_KP_APR = 27
RID_KI_APR = 28


F32_FIELDS = {
    "kp_asr": (RID_KP_ASR, "KP_ASR"),
    "ki_asr": (RID_KI_ASR, "KI_ASR"),
    "kp_apr": (RID_KP_APR, "KP_APR"),
    "ki_apr": (RID_KI_APR, "KI_APR"),
    "pmax": (RID_PMAX, "PMAX"),
    "vmax": (RID_VMAX, "VMAX"),
    "tmax": (RID_TMAX, "TMAX"),
    "acc": (RID_ACC, "ACC"),
    "dec": (RID_DEC, "DEC"),
    "max_spd": (RID_MAX_SPD, "MAX_SPD"),
}


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(description="Tune PID/high-impact Damiao registers via Python SDK")
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4340P")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")
    p.add_argument("--timeout-ms", type=int, default=1000)
    p.add_argument("--store", type=int, default=0)

    p.add_argument("--kp-asr", type=float)
    p.add_argument("--ki-asr", type=float)
    p.add_argument("--kp-apr", type=float)
    p.add_argument("--ki-apr", type=float)
    p.add_argument("--pmax", type=float)
    p.add_argument("--vmax", type=float)
    p.add_argument("--tmax", type=float)
    p.add_argument("--acc", type=float)
    p.add_argument("--dec", type=float)
    p.add_argument("--max-spd", type=float)

    args = p.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)

    print(
        f"channel={args.channel} model={args.model} motor_id=0x{motor_id:X} "
        f"feedback_id=0x{feedback_id:X}"
    )

    with Controller(args.channel) as ctrl:
        m = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            print("[before]")
            for _, (rid, name) in F32_FIELDS.items():
                v = m.get_register_f32(rid, args.timeout_ms)
                print(f"{name} (rid={rid}) = {v}")

            for k, (rid, name) in F32_FIELDS.items():
                value = getattr(args, k)
                if value is None:
                    continue
                print(f"write {name} (rid={rid}) <= {value}")
                m.write_register_f32(rid, value)

            if args.store:
                m.store_parameters()
                print("store_parameters sent")

            print("[after]")
            for _, (rid, name) in F32_FIELDS.items():
                v = m.get_register_f32(rid, args.timeout_ms)
                print(f"{name} (rid={rid}) = {v}")
        finally:
            m.close()


if __name__ == "__main__":
    main()
