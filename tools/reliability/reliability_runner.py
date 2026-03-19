#!/usr/bin/env python3
import argparse
import json
import re
import subprocess
import time
from pathlib import Path


SCAN_DONE_RE = re.compile(r"\[scan\]\s+done\s+vendor=(\w+)\s+hits=(\d+)")
HIT_RE = re.compile(r"\[hit\]\s+vendor=(\w+)\s+id=(\d+)")


def run_endurance(command: str, duration_sec: int, interval_sec: float, log_path: Path) -> int:
    start = time.time()
    iteration = 0
    ok = 0
    fail = 0
    records = []
    log_path.parent.mkdir(parents=True, exist_ok=True)

    while (time.time() - start) < duration_sec:
        t0 = time.time()
        proc = subprocess.run(
            command,
            shell=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        dt = time.time() - t0
        iteration += 1
        rec = {
            "iteration": iteration,
            "returncode": proc.returncode,
            "duration_sec": round(dt, 4),
            "output": proc.stdout,
        }
        records.append(rec)
        if proc.returncode == 0:
            ok += 1
        else:
            fail += 1
        if interval_sec > 0:
            time.sleep(interval_sec)

    summary = {
        "command": command,
        "duration_sec": duration_sec,
        "interval_sec": interval_sec,
        "iterations": iteration,
        "ok": ok,
        "fail": fail,
        "success_rate": 0.0 if iteration == 0 else round(ok / iteration, 4),
        "records": records,
    }
    log_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"Saved endurance report to: {log_path}")
    print(
        f"iterations={summary['iterations']} ok={summary['ok']} fail={summary['fail']} success_rate={summary['success_rate']}"
    )
    return 0 if fail == 0 else 2


def parse_scan_log(text: str) -> dict:
    hits_by_vendor = {}
    device_ids = {}

    for vendor, hits in SCAN_DONE_RE.findall(text):
        hits_by_vendor[vendor] = int(hits)

    for vendor, dev_id in HIT_RE.findall(text):
        device_ids.setdefault(vendor, set()).add(int(dev_id))

    return {
        "hits_by_vendor": hits_by_vendor,
        "device_ids": {k: sorted(v) for k, v in device_ids.items()},
    }


def compare_scan(left: Path, right: Path) -> int:
    left_text = left.read_text(encoding="utf-8", errors="replace")
    right_text = right.read_text(encoding="utf-8", errors="replace")
    l = parse_scan_log(left_text)
    r = parse_scan_log(right_text)

    print("LEFT :", json.dumps(l, ensure_ascii=False))
    print("RIGHT:", json.dumps(r, ensure_ascii=False))

    if l == r:
        print("PASS: scan outputs are consistent")
        return 0

    print("FAIL: scan outputs differ")
    return 3


def main() -> int:
    parser = argparse.ArgumentParser(description="motorbridge reliability helper")
    sub = parser.add_subparsers(dest="cmd", required=True)

    e = sub.add_parser("endurance", help="run one command repeatedly and record results")
    e.add_argument("--command", required=True, help="command to execute")
    e.add_argument("--duration-sec", type=int, default=600, help="total duration")
    e.add_argument("--interval-sec", type=float, default=0.5, help="sleep between iterations")
    e.add_argument(
        "--report",
        default="tools/reliability/reports/endurance.json",
        help="output JSON report path",
    )

    c = sub.add_parser("compare-scan", help="compare Linux/Windows scan logs")
    c.add_argument("--left-log", required=True, help="left scan log file")
    c.add_argument("--right-log", required=True, help="right scan log file")

    args = parser.parse_args()
    if args.cmd == "endurance":
        return run_endurance(
            command=args.command,
            duration_sec=args.duration_sec,
            interval_sec=args.interval_sec,
            log_path=Path(args.report),
        )
    return compare_scan(Path(args.left_log), Path(args.right_log))


if __name__ == "__main__":
    raise SystemExit(main())
