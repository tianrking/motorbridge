#!/usr/bin/env python3
"""Factory calibration web console (multi-vendor scan + ID workflows).

Backend is motor_cli (official kernel-facing CLI surface).
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

SCAN_VENDORS = ["damiao", "robstride", "myactuator", "hightorque", "hexfellow"]
SET_ID_VENDORS = {"damiao", "robstride"}

RE_DAMIAO_HIT = re.compile(r"\[hit\]\s+vendor=damiao\s+id=(\d+)\s+feedback_id=0x([0-9A-Fa-f]+)")
RE_ROBSTRIDE_HIT = re.compile(r"\[hit\]\s+vendor=robstride\s+id=(\d+)")
RE_MYACT_HIT = re.compile(r"\[hit\]\s+vendor=myactuator\s+id=(\d+)\s+feedback_id=0x([0-9A-Fa-f]+)")
RE_HIGHTORQUE_HIT = re.compile(r"\[hit\]\s+id=(\d+)")
RE_HEXFELLOW_HIT = re.compile(r"\[hit\]\s+vendor=hexfellow\s+node=(\d+)")


class CliRunner:
    def __init__(self, repo_root: Path) -> None:
        self.repo_root = repo_root
        self.release_bin = self.repo_root / "target" / "release" / "motor_cli"

    def _base_cmd(self) -> list[str]:
        if self.release_bin.exists():
            return [str(self.release_bin)]
        return ["cargo", "run", "-p", "motor_cli", "--release", "--"]

    def run(self, args: list[str]) -> tuple[int, str, str, list[str]]:
        cmd = self._base_cmd() + args
        proc = subprocess.run(
            cmd,
            cwd=self.repo_root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        return proc.returncode, proc.stdout, proc.stderr, cmd


class App:
    def __init__(self, repo_root: Path) -> None:
        self.repo_root = repo_root
        self.web_root = Path(__file__).resolve().parent
        self.runner = CliRunner(repo_root)

    @staticmethod
    def _to_int(v: Any, default: int) -> int:
        if v is None:
            return default
        if isinstance(v, int):
            return v
        s = str(v).strip()
        if not s:
            return default
        return int(s, 0)

    @staticmethod
    def _to_hex(n: int) -> str:
        return f"0x{n:X}"

    @staticmethod
    def _normalize_vendors(payload: dict[str, Any]) -> list[str]:
        raw = payload.get("vendors")
        if isinstance(raw, list):
            vals = [str(v).strip().lower() for v in raw if str(v).strip()]
        elif isinstance(raw, str):
            vals = [raw.strip().lower()] if raw.strip() else []
        else:
            v = str(payload.get("vendor", "")).strip().lower()
            vals = [v] if v else []

        if not vals:
            vals = ["damiao", "robstride"]
        if "all" in vals:
            return SCAN_VENDORS.copy()

        out: list[str] = []
        for v in vals:
            if v in SCAN_VENDORS and v not in out:
                out.append(v)
        return out

    @staticmethod
    def _dedupe_hits(hits: list[dict[str, Any]]) -> list[dict[str, Any]]:
        out: list[dict[str, Any]] = []
        seen: set[tuple[str, int]] = set()
        for h in hits:
            k = (str(h.get("vendor", "")), int(h.get("esc_id", 0)))
            if k in seen:
                continue
            seen.add(k)
            out.append(h)
        return out

    def _vendor_cfg(self, payload: dict[str, Any], vendor: str) -> dict[str, Any]:
        vc_all = payload.get("vendor_configs", {})
        vc: dict[str, Any] = {}
        if isinstance(vc_all, dict):
            maybe = vc_all.get(vendor, {})
            if isinstance(maybe, dict):
                vc = maybe

        if vendor == "damiao":
            return {
                "model": str(vc.get("model", payload.get("model_damiao", "4310"))),
                "start_id": self._to_int(vc.get("start_id", payload.get("start_id")), 1),
                "end_id": self._to_int(vc.get("end_id", payload.get("end_id")), 16),
                "feedback_base": self._to_int(
                    vc.get("feedback_base", payload.get("feedback_base")), 0x10
                ),
            }
        if vendor == "robstride":
            return {
                "model": str(vc.get("model", payload.get("model_robstride", "rs-00"))),
                "start_id": self._to_int(vc.get("start_id", payload.get("start_id")), 1),
                "end_id": self._to_int(vc.get("end_id", payload.get("end_id")), 16),
                "feedback_id": self._to_int(
                    vc.get("feedback_id", payload.get("robstride_feedback_id")), 0xFF
                ),
            }
        if vendor == "myactuator":
            return {
                "model": str(vc.get("model", payload.get("model_myactuator", "X8"))),
                "start_id": self._to_int(vc.get("start_id", payload.get("start_id")), 1),
                "end_id": self._to_int(vc.get("end_id", payload.get("end_id")), 16),
            }
        if vendor == "hightorque":
            return {
                "model": "hightorque",
                "start_id": self._to_int(vc.get("start_id", payload.get("start_id")), 1),
                "end_id": self._to_int(vc.get("end_id", payload.get("end_id")), 16),
            }
        if vendor == "hexfellow":
            return {
                "model": str(vc.get("model", payload.get("model_hexfellow", "hexfellow"))),
                "start_id": self._to_int(vc.get("start_id", payload.get("start_id")), 1),
                "end_id": self._to_int(vc.get("end_id", payload.get("end_id")), 16),
            }
        return {}

    def _scan_damiao(
        self,
        channel: str,
        model: str,
        start_id: int,
        end_id: int,
        feedback_base: int,
    ) -> dict[str, Any]:
        rc, out, err, cmd = self.runner.run(
            [
                "--vendor",
                "damiao",
                "--channel",
                channel,
                "--model",
                model,
                "--feedback-id",
                self._to_hex(feedback_base),
                "--mode",
                "scan",
                "--start-id",
                self._to_hex(start_id),
                "--end-id",
                self._to_hex(end_id),
            ]
        )
        hits = []
        for m in RE_DAMIAO_HIT.finditer(out):
            esc = int(m.group(1), 10)
            mst = int(m.group(2), 16)
            hits.append(
                {
                    "vendor": "damiao",
                    "probe": esc,
                    "esc_id": esc,
                    "mst_id": mst,
                    "feedback_probe": mst,
                    "model": model,
                }
            )
        return {"ok": rc == 0, "hits": hits, "code": rc, "stdout": out, "stderr": err, "command": " ".join(cmd)}

    def _scan_robstride(
        self,
        channel: str,
        model: str,
        start_id: int,
        end_id: int,
        feedback_id: int,
    ) -> dict[str, Any]:
        rc, out, err, cmd = self.runner.run(
            [
                "--vendor",
                "robstride",
                "--channel",
                channel,
                "--model",
                model,
                "--feedback-id",
                self._to_hex(feedback_id),
                "--mode",
                "scan",
                "--start-id",
                self._to_hex(start_id),
                "--end-id",
                self._to_hex(end_id),
            ]
        )
        hits = []
        for m in RE_ROBSTRIDE_HIT.finditer(out):
            esc = int(m.group(1), 10)
            hits.append(
                {
                    "vendor": "robstride",
                    "probe": esc,
                    "esc_id": esc,
                    "mst_id": feedback_id,
                    "feedback_probe": feedback_id,
                    "model": model,
                }
            )
        return {"ok": rc == 0, "hits": hits, "code": rc, "stdout": out, "stderr": err, "command": " ".join(cmd)}

    def _scan_myactuator(self, channel: str, model: str, start_id: int, end_id: int) -> dict[str, Any]:
        rc, out, err, cmd = self.runner.run(
            [
                "--vendor",
                "myactuator",
                "--channel",
                channel,
                "--model",
                model,
                "--mode",
                "scan",
                "--start-id",
                self._to_hex(start_id),
                "--end-id",
                self._to_hex(end_id),
            ]
        )
        hits = []
        for m in RE_MYACT_HIT.finditer(out):
            esc = int(m.group(1), 10)
            mst = int(m.group(2), 16)
            hits.append(
                {
                    "vendor": "myactuator",
                    "probe": esc,
                    "esc_id": esc,
                    "mst_id": mst,
                    "feedback_probe": mst,
                    "model": model,
                }
            )
        return {"ok": rc == 0, "hits": hits, "code": rc, "stdout": out, "stderr": err, "command": " ".join(cmd)}

    def _scan_hightorque(self, channel: str, start_id: int, end_id: int) -> dict[str, Any]:
        rc, out, err, cmd = self.runner.run(
            [
                "--vendor",
                "hightorque",
                "--channel",
                channel,
                "--mode",
                "scan",
                "--start-id",
                self._to_hex(start_id),
                "--end-id",
                self._to_hex(end_id),
            ]
        )
        hits = []
        for m in RE_HIGHTORQUE_HIT.finditer(out):
            esc = int(m.group(1), 10)
            hits.append(
                {
                    "vendor": "hightorque",
                    "probe": esc,
                    "esc_id": esc,
                    "mst_id": 0x01,
                    "feedback_probe": 0x01,
                    "model": "hightorque",
                }
            )
        return {"ok": rc == 0, "hits": hits, "code": rc, "stdout": out, "stderr": err, "command": " ".join(cmd)}

    def _scan_hexfellow(self, channel: str, model: str, start_id: int, end_id: int) -> dict[str, Any]:
        rc, out, err, cmd = self.runner.run(
            [
                "--vendor",
                "hexfellow",
                "--transport",
                "socketcanfd",
                "--channel",
                channel,
                "--model",
                model,
                "--mode",
                "scan",
                "--start-id",
                self._to_hex(start_id),
                "--end-id",
                self._to_hex(end_id),
            ]
        )
        hits = []
        for m in RE_HEXFELLOW_HIT.finditer(out):
            esc = int(m.group(1), 10)
            hits.append(
                {
                    "vendor": "hexfellow",
                    "probe": esc,
                    "esc_id": esc,
                    "mst_id": 0,
                    "feedback_probe": 0,
                    "model": model,
                }
            )
        return {"ok": rc == 0, "hits": hits, "code": rc, "stdout": out, "stderr": err, "command": " ".join(cmd)}

    def _run_scan(self, payload: dict[str, Any]) -> dict[str, Any]:
        channel = str(payload.get("channel", "can0"))
        vendors = self._normalize_vendors(payload)
        if not vendors:
            return {"ok": False, "error": "no vendor selected"}

        results = []
        for v in vendors:
            cfg = self._vendor_cfg(payload, v)
            if int(cfg.get("start_id", 1)) > int(cfg.get("end_id", 16)):
                results.append(
                    {
                        "ok": False,
                        "hits": [],
                        "code": 1,
                        "stdout": "",
                        "stderr": f"[scan] vendor={v} invalid range: start_id > end_id",
                        "command": f"vendor={v} (skipped)",
                    }
                )
                continue
            if v == "damiao":
                results.append(
                    self._scan_damiao(
                        channel,
                        str(cfg["model"]),
                        int(cfg["start_id"]),
                        int(cfg["end_id"]),
                        int(cfg["feedback_base"]),
                    )
                )
            elif v == "robstride":
                results.append(
                    self._scan_robstride(
                        channel,
                        str(cfg["model"]),
                        int(cfg["start_id"]),
                        int(cfg["end_id"]),
                        int(cfg["feedback_id"]),
                    )
                )
            elif v == "myactuator":
                results.append(
                    self._scan_myactuator(
                        channel,
                        str(cfg["model"]),
                        int(cfg["start_id"]),
                        int(cfg["end_id"]),
                    )
                )
            elif v == "hightorque":
                results.append(
                    self._scan_hightorque(
                        channel,
                        int(cfg["start_id"]),
                        int(cfg["end_id"]),
                    )
                )
            elif v == "hexfellow":
                results.append(
                    self._scan_hexfellow(
                        channel,
                        str(cfg["model"]),
                        int(cfg["start_id"]),
                        int(cfg["end_id"]),
                    )
                )

        all_hits: list[dict[str, Any]] = []
        ok = True
        code = 0
        stdout_parts = []
        stderr_parts = []
        commands = []
        for r in results:
            all_hits.extend(r["hits"])
            ok = ok and bool(r["ok"])
            code = code or int(r["code"])
            if r["stdout"]:
                stdout_parts.append(r["stdout"].strip())
            if r["stderr"]:
                stderr_parts.append(r["stderr"].strip())
            commands.append(r["command"])

        return {
            "ok": ok,
            "hits": self._dedupe_hits(all_hits),
            "code": code,
            "stdout": "\n\n".join(stdout_parts),
            "stderr": "\n\n".join(stderr_parts),
            "command": "\n".join(commands),
        }

    def _run_verify(self, payload: dict[str, Any]) -> dict[str, Any]:
        channel = str(payload.get("channel", "can0"))
        vendor = str(payload.get("vendor", "damiao")).strip().lower()
        model = str(payload.get("model", ""))
        motor_id = self._to_int(payload.get("motor_id"), 1)
        feedback_id = self._to_int(payload.get("feedback_id"), 0x11)

        # verify by single-id scan/ping according to vendor
        if vendor == "damiao":
            ret = self._scan_damiao(channel, model or "4310", motor_id, motor_id, feedback_id)
            hits = ret["hits"]
            matched = hits[0] if hits else None
            return {
                "ok": ret["ok"] and matched is not None,
                "esc_id": matched["esc_id"] if matched else None,
                "mst_id": matched["mst_id"] if matched else None,
                "code": ret["code"],
                "stdout": ret["stdout"],
                "stderr": ret["stderr"],
                "command": ret["command"],
            }
        if vendor == "robstride":
            ret = self._scan_robstride(channel, model or "rs-00", motor_id, motor_id, feedback_id)
            hits = ret["hits"]
            matched = hits[0] if hits else None
            return {
                "ok": ret["ok"] and matched is not None,
                "esc_id": matched["esc_id"] if matched else None,
                "mst_id": matched["mst_id"] if matched else None,
                "code": ret["code"],
                "stdout": ret["stdout"],
                "stderr": ret["stderr"],
                "command": ret["command"],
            }
        if vendor == "myactuator":
            ret = self._scan_myactuator(channel, model or "X8", motor_id, motor_id)
            hits = ret["hits"]
            matched = hits[0] if hits else None
            return {
                "ok": ret["ok"] and matched is not None,
                "esc_id": matched["esc_id"] if matched else None,
                "mst_id": matched["mst_id"] if matched else None,
                "code": ret["code"],
                "stdout": ret["stdout"],
                "stderr": ret["stderr"],
                "command": ret["command"],
            }
        if vendor == "hightorque":
            ret = self._scan_hightorque(channel, motor_id, motor_id)
            hits = ret["hits"]
            matched = hits[0] if hits else None
            return {
                "ok": ret["ok"] and matched is not None,
                "esc_id": matched["esc_id"] if matched else None,
                "mst_id": matched["mst_id"] if matched else None,
                "code": ret["code"],
                "stdout": ret["stdout"],
                "stderr": ret["stderr"],
                "command": ret["command"],
            }
        if vendor == "hexfellow":
            ret = self._scan_hexfellow(channel, model or "hexfellow", motor_id, motor_id)
            hits = ret["hits"]
            matched = hits[0] if hits else None
            return {
                "ok": ret["ok"] and matched is not None,
                "esc_id": matched["esc_id"] if matched else None,
                "mst_id": matched["mst_id"] if matched else None,
                "code": ret["code"],
                "stdout": ret["stdout"],
                "stderr": ret["stderr"],
                "command": ret["command"],
            }

        return {"ok": False, "error": f"unsupported vendor for verify: {vendor}"}

    def _run_set_id(self, payload: dict[str, Any]) -> dict[str, Any]:
        channel = str(payload.get("channel", "can0"))
        vendor = str(payload.get("vendor", "damiao")).strip().lower()
        model = str(payload.get("model", "4310"))
        motor_id = self._to_int(payload.get("motor_id"), 1)
        feedback_id = self._to_int(payload.get("feedback_id"), 0x11)
        new_motor_id = self._to_int(payload.get("new_motor_id"), motor_id)
        new_feedback_id = self._to_int(payload.get("new_feedback_id"), feedback_id)
        store = 1 if payload.get("store", True) else 0
        verify = 1 if payload.get("verify", True) else 0
        timeout_ms = self._to_int(payload.get("timeout_ms"), 1000)

        if vendor not in SET_ID_VENDORS:
            return {
                "ok": False,
                "error": f"set-id is not supported in UI for vendor '{vendor}' yet",
            }

        if vendor == "damiao":
            rc, out, err, cmd = self.runner.run(
                [
                    "--vendor",
                    "damiao",
                    "--channel",
                    channel,
                    "--model",
                    model,
                    "--motor-id",
                    self._to_hex(motor_id),
                    "--feedback-id",
                    self._to_hex(feedback_id),
                    "--set-motor-id",
                    self._to_hex(new_motor_id),
                    "--set-feedback-id",
                    self._to_hex(new_feedback_id),
                    "--store",
                    str(store),
                    "--verify-id",
                    str(verify),
                    "--verify-timeout-ms",
                    str(timeout_ms),
                ]
            )
            return {"ok": rc == 0, "code": rc, "stdout": out, "stderr": err, "command": " ".join(cmd)}

        # robstride
        rc, out, err, cmd = self.runner.run(
            [
                "--vendor",
                "robstride",
                "--channel",
                channel,
                "--model",
                model,
                "--motor-id",
                self._to_hex(motor_id),
                "--feedback-id",
                self._to_hex(feedback_id),
                "--set-motor-id",
                self._to_hex(new_motor_id),
                "--store",
                str(store),
            ]
        )
        if rc == 0 and verify:
            vret = self._scan_robstride(channel, model or "rs-00", new_motor_id, new_motor_id, feedback_id)
            rc = 0 if (vret["ok"] and vret["hits"]) else 1
            out = (out.strip() + "\n" + str(vret.get("stdout", "")).strip()).strip()
            err = (err.strip() + "\n" + str(vret.get("stderr", "")).strip()).strip()
            cmd = cmd + ["&&", vret.get("command", "")]

        return {"ok": rc == 0, "code": rc, "stdout": out, "stderr": err, "command": " ".join([c for c in cmd if c])}

    def handle_api(self, path: str, payload: dict[str, Any]) -> tuple[int, dict[str, Any]]:
        try:
            if path == "/api/health":
                return 200, {
                    "ok": True,
                    "repo_root": str(self.repo_root),
                    "mode": "release-bin" if self.runner.release_bin.exists() else "cargo-run",
                    "backend": "motor_cli",
                    "scan_vendors": SCAN_VENDORS,
                    "set_id_vendors": sorted(SET_ID_VENDORS),
                }
            if path == "/api/scan":
                return 200, self._run_scan(payload)
            if path == "/api/verify":
                return 200, self._run_verify(payload)
            if path == "/api/set-id":
                return 200, self._run_set_id(payload)
            return 404, {"ok": False, "error": f"unknown api path: {path}"}
        except Exception as e:  # noqa: BLE001
            return 500, {"ok": False, "error": str(e)}


class Handler(BaseHTTPRequestHandler):
    app: App

    def _send_json(self, code: int, data: dict[str, Any]) -> None:
        raw = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(raw)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(raw)

    def _send_file(self, file_path: Path, content_type: str) -> None:
        if not file_path.exists():
            self.send_error(404)
            return
        raw = file_path.read_bytes()
        self.send_response(200)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def do_GET(self) -> None:  # noqa: N802
        if self.path == "/" or self.path.startswith("/index.html"):
            self._send_file(self.app.web_root / "index.html", "text/html; charset=utf-8")
            return
        if self.path == "/api/health":
            code, data = self.app.handle_api("/api/health", {})
            self._send_json(code, data)
            return
        self.send_error(404)

    def do_POST(self) -> None:  # noqa: N802
        n = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(n) if n > 0 else b"{}"
        try:
            payload = json.loads(body.decode("utf-8")) if body else {}
            if not isinstance(payload, dict):
                payload = {}
        except json.JSONDecodeError:
            self._send_json(400, {"ok": False, "error": "invalid json"})
            return

        code, data = self.app.handle_api(self.path, payload)
        self._send_json(code, data)

    def log_message(self, fmt: str, *args: Any) -> None:
        sys.stderr.write(f"[factory-ui] {self.address_string()} - {fmt % args}\n")


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Factory web UI for multi-vendor ID workflows")
    p.add_argument("--bind", default="127.0.0.1", help="bind address")
    p.add_argument("--port", type=int, default=18100, help="http listen port")
    p.add_argument(
        "--repo-root",
        default=str(Path(__file__).resolve().parents[2]),
        help="path to rust_dm repo root",
    )
    return p.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    app = App(repo_root)
    Handler.app = app

    server = ThreadingHTTPServer((args.bind, args.port), Handler)
    print(f"[factory-ui] listening at http://{args.bind}:{args.port}")
    print(f"[factory-ui] repo_root={repo_root}")
    print("[factory-ui] backend=motor_cli (release-bin preferred, fallback cargo run)")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
