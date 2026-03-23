#!/usr/bin/env python3
import argparse
import json
import re
import subprocess
import time
from pathlib import Path


SCAN_DONE_RE = re.compile(r"\[scan\]\s+done\s+vendor=(\w+)\s+hits=(\d+)")
HIT_RE = re.compile(r"\[hit\]\s+vendor=(\w+)\s+id=(\d+)")


def load_endurance_template(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"template must be a JSON object: {path}")
    return data


def run_endurance(
    command: str,
    duration_sec: int,
    interval_sec: float,
    log_path: Path,
    min_success_rate: float | None = None,
    max_fail: int | None = None,
) -> int:
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
        "thresholds": {
            "min_success_rate": min_success_rate,
            "max_fail": max_fail,
        },
        "iterations": iteration,
        "ok": ok,
        "fail": fail,
        "success_rate": 0.0 if iteration == 0 else round(ok / iteration, 4),
        "records": records,
    }
    pass_ok = True
    reasons = []
    if max_fail is not None and fail > max_fail:
        pass_ok = False
        reasons.append(f"fail={fail} > max_fail={max_fail}")
    if min_success_rate is not None and summary["success_rate"] < min_success_rate:
        pass_ok = False
        reasons.append(
            f"success_rate={summary['success_rate']} < min_success_rate={min_success_rate}"
        )
    summary["pass"] = pass_ok
    summary["fail_reasons"] = reasons
    log_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"Saved endurance report to: {log_path}")
    print(
        f"iterations={summary['iterations']} ok={summary['ok']} fail={summary['fail']} success_rate={summary['success_rate']}"
    )
    if pass_ok:
        print("PASS: endurance thresholds satisfied")
        return 0
    print("FAIL: endurance thresholds not satisfied")
    for item in reasons:
        print(f"- {item}")
    return 2


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


def _split_csv(text: str | None) -> list[str]:
    if not text:
        return []
    return [x.strip() for x in text.split(",") if x.strip()]


def _ids_match(left_ids: set[int], right_ids: set[int], mode: str) -> bool:
    if mode == "exact":
        return left_ids == right_ids
    if mode == "left-subset":
        return left_ids.issubset(right_ids)
    if mode == "right-subset":
        return right_ids.issubset(left_ids)
    if mode == "intersect-nonempty":
        if not left_ids and not right_ids:
            return True
        return bool(left_ids.intersection(right_ids))
    raise ValueError(f"unsupported id mode: {mode}")


def compare_scan(
    left: Path,
    right: Path,
    vendors: list[str],
    allow_hit_delta: int,
    id_mode: str,
) -> int:
    left_text = left.read_text(encoding="utf-8", errors="replace")
    right_text = right.read_text(encoding="utf-8", errors="replace")
    l = parse_scan_log(left_text)
    r = parse_scan_log(right_text)

    print("LEFT :", json.dumps(l, ensure_ascii=False))
    print("RIGHT:", json.dumps(r, ensure_ascii=False))
    selected = set(vendors) if vendors else (
        set(l["hits_by_vendor"]).union(r["hits_by_vendor"]).union(l["device_ids"]).union(r["device_ids"])
    )
    if not selected:
        print("PASS: no vendors found in either log")
        return 0

    ok = True
    for vendor in sorted(selected):
        left_ids = set(l["device_ids"].get(vendor, []))
        right_ids = set(r["device_ids"].get(vendor, []))
        left_hits = l["hits_by_vendor"].get(vendor, len(left_ids))
        right_hits = r["hits_by_vendor"].get(vendor, len(right_ids))
        hit_ok = abs(left_hits - right_hits) <= allow_hit_delta
        id_ok = _ids_match(left_ids, right_ids, id_mode)
        status = "PASS" if (hit_ok and id_ok) else "FAIL"
        print(
            f"[{status}] vendor={vendor} left_hits={left_hits} right_hits={right_hits} "
            f"allow_hit_delta={allow_hit_delta} id_mode={id_mode} "
            f"left_ids={sorted(left_ids)} right_ids={sorted(right_ids)}"
        )
        if not (hit_ok and id_ok):
            ok = False

    if ok:
        print("PASS: scan outputs are consistent under configured rules")
        return 0
    print("FAIL: scan outputs differ under configured rules")
    return 3


def main() -> int:
    parser = argparse.ArgumentParser(description="motorbridge reliability helper")
    sub = parser.add_subparsers(dest="cmd", required=True)

    e = sub.add_parser("endurance", help="run one command repeatedly and record results")
    e.add_argument("--template", help="optional JSON template for endurance config")
    e.add_argument("--command", help="command to execute")
    e.add_argument("--duration-sec", type=int, help="total duration")
    e.add_argument("--interval-sec", type=float, help="sleep between iterations")
    e.add_argument(
        "--report",
        help="output JSON report path",
    )
    e.add_argument(
        "--min-success-rate",
        type=float,
        help="minimum acceptable success rate (0.0~1.0)",
    )
    e.add_argument(
        "--max-fail",
        type=int,
        help="maximum acceptable fail count",
    )
    e.add_argument(
        "--default-report",
        default="tools/reliability/reports/endurance.json",
        help="output JSON report path",
    )

    c = sub.add_parser("compare-scan", help="compare Linux/Windows scan logs")
    c.add_argument("--left-log", required=True, help="left scan log file")
    c.add_argument("--right-log", required=True, help="right scan log file")
    c.add_argument(
        "--vendors",
        default="",
        help="comma-separated vendor subset to compare; default compares all vendors found",
    )
    c.add_argument(
        "--allow-hit-delta",
        type=int,
        default=0,
        help="allow absolute hit-count delta per vendor",
    )
    c.add_argument(
        "--id-mode",
        choices=["exact", "left-subset", "right-subset", "intersect-nonempty"],
        default="exact",
        help="ID set comparison mode",
    )

    args = parser.parse_args()
    if args.cmd == "endurance":
        template = {}
        if args.template:
            template = load_endurance_template(Path(args.template))

        command = args.command or template.get("command")
        if not command:
            raise SystemExit("endurance requires --command or --template with `command`")

        duration_sec = (
            args.duration_sec
            if args.duration_sec is not None
            else int(template.get("duration_sec", 600))
        )
        interval_sec = (
            args.interval_sec
            if args.interval_sec is not None
            else float(template.get("interval_sec", 0.5))
        )
        report = args.report or template.get("report") or args.default_report
        thresholds = template.get("thresholds", {}) if isinstance(template.get("thresholds", {}), dict) else {}
        min_success_rate = (
            args.min_success_rate
            if args.min_success_rate is not None
            else thresholds.get("min_success_rate")
        )
        max_fail = args.max_fail if args.max_fail is not None else thresholds.get("max_fail")
        return run_endurance(
            command=command,
            duration_sec=duration_sec,
            interval_sec=interval_sec,
            log_path=Path(report),
            min_success_rate=min_success_rate,
            max_fail=max_fail,
        )
    return compare_scan(
        left=Path(args.left_log),
        right=Path(args.right_log),
        vendors=_split_csv(args.vendors),
        allow_hit_delta=args.allow_hit_delta,
        id_mode=args.id_mode,
    )


if __name__ == "__main__":
    raise SystemExit(main())
