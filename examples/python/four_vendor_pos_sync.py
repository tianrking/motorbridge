#!/usr/bin/env python3
import argparse
import shlex
import subprocess
import sys
import time
from typing import List, Optional, Tuple


def cmd_to_text(cmd: List[str]) -> str:
    return " ".join(shlex.quote(x) for x in cmd)


def parse_int_auto(raw: str) -> int:
    return int(raw, 0)


def format_id(value: int) -> str:
    return hex(value)


def default_feedback_id(vendor: str, motor_id: int) -> Optional[int]:
    if vendor == "damiao":
        return motor_id + 0x10
    if vendor == "myactuator":
        return 0x240 + motor_id
    if vendor == "robstride":
        return 0xFF
    return None


def normalize_vendor(raw: str) -> str:
    v = raw.strip().lower()
    alias = {
        "dm": "damiao",
        "ht": "hightorque",
        "my": "myactuator",
        "ma": "myactuator",
        "rs": "robstride",
    }
    v = alias.get(v, v)
    supported = {"damiao", "myactuator", "hightorque", "robstride"}
    if v not in supported:
        raise ValueError(f"unsupported vendor '{raw}'")
    return v


def parse_targets(raw_targets: List[str]) -> List[Tuple[str, int, Optional[int]]]:
    if not raw_targets:
        raise ValueError("missing target motors")

    # Compact form: vendor:motor_id[:feedback_id]
    if any(":" in t for t in raw_targets):
        parsed: List[Tuple[str, int, Optional[int]]] = []
        for tok in raw_targets:
            parts = tok.split(":")
            if len(parts) < 2 or len(parts) > 3:
                raise ValueError(
                    f"invalid target token '{tok}', expected vendor:motor_id[:feedback_id]"
                )
            vendor = normalize_vendor(parts[0])
            motor_id = parse_int_auto(parts[1])
            feedback = parse_int_auto(parts[2]) if len(parts) == 3 else None
            parsed.append((vendor, motor_id, feedback))
        return parsed

    # Pair form: vendor motor_id vendor motor_id ...
    if len(raw_targets) % 2 != 0:
        raise ValueError("target list must be pairs: vendor motor_id vendor motor_id ...")

    parsed: List[Tuple[str, int, Optional[int]]] = []
    i = 0
    while i < len(raw_targets):
        vendor = normalize_vendor(raw_targets[i])
        motor_id = parse_int_auto(raw_targets[i + 1])
        parsed.append((vendor, motor_id, None))
        i += 2
    return parsed


def parse_model_map(raw: str) -> dict:
    # format: "0x01=4340P,0x07=4310"
    result = {}
    text = (raw or "").strip()
    if not text:
        return result
    for item in text.split(","):
        pair = item.strip()
        if not pair:
            continue
        if "=" not in pair:
            raise ValueError(
                f"invalid --damiao-model-by-id entry '{pair}', expected id=model"
            )
        left, right = pair.split("=", 1)
        mid = parse_int_auto(left.strip())
        model = right.strip()
        if not model:
            raise ValueError(
                f"invalid --damiao-model-by-id entry '{pair}', model is empty"
            )
        result[mid] = model
    return result


def build_vendor_command(
    args: argparse.Namespace, vendor: str, motor_id: int, feedback_id: Optional[int]
) -> Tuple[str, List[str]]:
    common = ["cargo", "run", "-p", "motor_cli", "--release", "--"]
    name = f"{vendor}-{motor_id}"

    if vendor == "damiao":
        fid = feedback_id if feedback_id is not None else default_feedback_id(vendor, motor_id)
        dm_model = args.damiao_model_by_id.get(motor_id, args.damiao_model)
        cmd = common + [
            "--vendor",
            "damiao",
            "--channel",
            args.channel,
            "--model",
            dm_model,
            "--motor-id",
            format_id(motor_id),
            "--feedback-id",
            format_id(fid),
            "--mode",
            "pos-vel",
            "--pos",
            str(args.pos),
            "--vlim",
            str(args.damiao_vlim),
            "--loop",
            str(args.loop),
            "--dt-ms",
            str(args.dt_ms),
        ]
        return name, cmd

    if vendor == "myactuator":
        fid = feedback_id if feedback_id is not None else default_feedback_id(vendor, motor_id)
        cmd = common + [
            "--vendor",
            "myactuator",
            "--channel",
            args.channel,
            "--model",
            args.myactuator_model,
            "--motor-id",
            str(motor_id),
            "--feedback-id",
            format_id(fid),
            "--mode",
            "pos",
            "--pos",
            str(args.pos),
            "--max-speed",
            str(args.myactuator_max_speed),
            "--loop",
            str(args.loop),
            "--dt-ms",
            str(args.dt_ms),
        ]
        return name, cmd

    if vendor == "hightorque":
        if args.hightorque_mode == "mit":
            cmd = common + [
                "--vendor",
                "hightorque",
                "--channel",
                args.channel,
                "--motor-id",
                str(motor_id),
                "--mode",
                "mit",
                "--pos",
                str(args.pos),
                "--vel",
                str(args.hightorque_vel),
                "--tau",
                str(args.hightorque_tau),
                "--loop",
                str(args.loop),
                "--dt-ms",
                str(args.dt_ms),
            ]
        else:
            cmd = common + [
                "--vendor",
                "hightorque",
                "--channel",
                args.channel,
                "--motor-id",
                str(motor_id),
                "--mode",
                "pos",
                "--pos",
                str(args.pos),
                "--tau",
                str(args.hightorque_tau),
                "--loop",
                str(args.loop),
                "--dt-ms",
                str(args.dt_ms),
            ]
        return name, cmd

    if vendor == "robstride":
        fid = feedback_id if feedback_id is not None else default_feedback_id(vendor, motor_id)
        cmd = common + [
            "--vendor",
            "robstride",
            "--channel",
            args.channel,
            "--model",
            args.robstride_model,
            "--motor-id",
            str(motor_id),
            "--feedback-id",
            format_id(fid),
            "--mode",
            "mit",
            "--pos",
            str(args.pos),
            "--vel",
            "0",
            "--kp",
            str(args.robstride_kp),
            "--kd",
            str(args.robstride_kd),
            "--tau",
            "0",
            "--loop",
            str(args.loop),
            "--dt-ms",
            str(args.dt_ms),
        ]
        return name, cmd

    raise ValueError(f"unsupported vendor '{vendor}'")


def build_commands(args: argparse.Namespace) -> List[Tuple[str, List[str]]]:
    parsed = parse_targets(args.targets)
    return [build_vendor_command(args, v, mid, fid) for v, mid, fid in parsed]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Run N motors to one position concurrently.\n"
            "targets supports:\n"
            "  1) pairs: vendor motor_id vendor motor_id ...\n"
            "  2) compact: vendor:motor_id[:feedback_id] ..."
        ),
        formatter_class=argparse.RawTextHelpFormatter,
    )
    parser.add_argument("targets", nargs="+")
    parser.add_argument("--pos", type=float, required=True, help="Target position in rad")
    parser.add_argument("--channel", default="can0")
    parser.add_argument("--loop", type=int, default=200)
    parser.add_argument("--dt-ms", type=int, default=20)
    parser.add_argument(
        "--stagger-ms",
        type=int,
        default=50,
        help="Delay between launching each motor command (ms)",
    )

    parser.add_argument("--damiao-model", default="4340P")
    parser.add_argument(
        "--damiao-model-by-id",
        default="",
        help="Per-ID Damiao model override, e.g. 0x01=4340P,0x07=4310",
    )
    parser.add_argument("--damiao-vlim", type=float, default=1.0)

    parser.add_argument("--myactuator-model", default="X8")
    parser.add_argument("--myactuator-max-speed", type=float, default=1.0)

    parser.add_argument("--hightorque-mode", choices=["mit", "pos"], default="mit")
    parser.add_argument("--hightorque-vel", type=float, default=0.8)
    parser.add_argument("--hightorque-tau", type=float, default=0.8)

    parser.add_argument("--robstride-model", default="rs-00")
    parser.add_argument("--robstride-kp", type=float, default=8.0)
    parser.add_argument("--robstride-kd", type=float, default=0.2)

    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print generated commands without executing",
    )
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    try:
        args.damiao_model_by_id = parse_model_map(args.damiao_model_by_id)
        commands = build_commands(args)
    except ValueError as e:
        print(f"argument error: {e}", file=sys.stderr)
        return 2

    print("== Multi-motor position sync ==")
    print(
        f"channel={args.channel} pos={args.pos}rad loop={args.loop} "
        f"dt_ms={args.dt_ms} stagger_ms={args.stagger_ms}"
    )
    for name, cmd in commands:
        print(f"[{name}] {cmd_to_text(cmd)}")

    if args.dry_run:
        return 0

    procs = []
    for idx, (name, cmd) in enumerate(commands):
        p = subprocess.Popen(cmd)
        procs.append((name, p))
        if args.stagger_ms > 0 and idx < len(commands) - 1:
            time.sleep(args.stagger_ms / 1000.0)

    failed = False
    for name, p in procs:
        rc = p.wait()
        if rc != 0:
            failed = True
            print(f"[{name}] exit code {rc}", file=sys.stderr)

    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
