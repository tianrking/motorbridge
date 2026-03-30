#!/usr/bin/env python3
from __future__ import annotations

import argparse
import asyncio
import contextlib
import json
import threading
import time
from dataclasses import dataclass, field
from typing import Any, Dict

from motorbridge import Controller, Mode


def _parse_int(v: str) -> int:
    return int(v, 0)


@dataclass
class MotorCfg:
    pos: float = 0.0
    active: bool = True
    # For POS_VEL motors (Damiao / MyActuator), vlim is the main speed limiter.
    # Slightly increased default to make motion faster out-of-box.
    vlim: float = 1.8
    vel: float = 0.0
    # For MIT motor (RobStride), kp/kd shape tracking speed/damping.
    # Slightly increased defaults for quicker response.
    kp: float = 3.0
    kd: float = 2.8
    tau: float = 0.0
    dir_sign: float = -1.0


@dataclass
class SharedState:
    running: bool = True
    enabled: bool = False
    last_error: str = ""
    tick_hz: float = 50.0
    motors: Dict[str, MotorCfg] = field(
        default_factory=lambda: {
            # Tuned defaults: a bit faster than previous profile, still conservative.
            "dm1": MotorCfg(vlim=1.8),
            "dm2": MotorCfg(vlim=1.8),
            "my": MotorCfg(vlim=2.0),
            "rs": MotorCfg(kp=3.0, kd=2.8, dir_sign=-1.0),
        }
    )


class Bridge:
    def __init__(self, args: argparse.Namespace) -> None:
        self.args = args
        self.state = SharedState(tick_hz=1000.0 / max(1, args.dt_ms))
        self.lock = threading.Lock()
        self.stop_evt = threading.Event()
        self.thread = threading.Thread(target=self._control_loop, daemon=True)
        self.thread.start()

    def _snapshot(self) -> Dict[str, Any]:
        with self.lock:
            return {
                "running": self.state.running,
                "enabled": self.state.enabled,
                "last_error": self.state.last_error,
                "tick_hz": self.state.tick_hz,
                "motors": {
                    k: {
                        "pos": v.pos,
                        "active": v.active,
                        "vlim": v.vlim,
                        "vel": v.vel,
                        "kp": v.kp,
                        "kd": v.kd,
                        "tau": v.tau,
                        "dir_sign": v.dir_sign,
                    }
                    for k, v in self.state.motors.items()
                },
            }

    def _set_error(self, text: str) -> None:
        with self.lock:
            self.state.last_error = text

    def apply_message(self, msg: Dict[str, Any]) -> Dict[str, Any]:
        op = str(msg.get("op", "")).strip().lower()
        if op == "ping":
            return {"ok": True, "pong": True}
        if op == "get_state":
            return {"ok": True, "state": self._snapshot()}
        if op == "set_target":
            motor = str(msg.get("motor", "")).strip()
            pos = float(msg.get("pos", 0.0))
            with self.lock:
                if motor not in self.state.motors:
                    return {"ok": False, "error": f"unknown motor '{motor}'"}
                self.state.motors[motor].pos = pos
            return {"ok": True, "motor": motor, "pos": pos}
        if op == "set_targets":
            with self.lock:
                for motor, val in msg.items():
                    if motor in self.state.motors:
                        self.state.motors[motor].pos = float(val)
            return {"ok": True}
        if op == "set_active":
            motor = str(msg.get("motor", "")).strip()
            enabled = bool(msg.get("enabled", True))
            with self.lock:
                if motor not in self.state.motors:
                    return {"ok": False, "error": f"unknown motor '{motor}'"}
                self.state.motors[motor].active = enabled
            return {"ok": True, "motor": motor, "active": enabled}
        if op == "set_enabled":
            enabled = bool(msg.get("enabled", True))
            with self.lock:
                self.state.enabled = enabled
            return {"ok": True, "enabled": enabled}
        if op == "set_param":
            motor = str(msg.get("motor", "")).strip()
            key = str(msg.get("key", "")).strip()
            val = float(msg.get("value", 0.0))
            with self.lock:
                if motor not in self.state.motors:
                    return {"ok": False, "error": f"unknown motor '{motor}'"}
                cfg = self.state.motors[motor]
                if not hasattr(cfg, key):
                    return {"ok": False, "error": f"unknown param '{key}'"}
                setattr(cfg, key, val)
            return {"ok": True, "motor": motor, "key": key, "value": val}
        return {"ok": False, "error": f"unsupported op '{op}'"}

    def _control_loop(self) -> None:
        a = self.args
        dm1_id, dm1_fid = _parse_int(a.dm1_id), _parse_int(a.dm1_fid)
        dm2_id, dm2_fid = _parse_int(a.dm2_id), _parse_int(a.dm2_fid)
        my_id, my_fid = _parse_int(a.my_id), _parse_int(a.my_fid)
        rs_id, rs_fid = _parse_int(a.rs_id), _parse_int(a.rs_fid)

        while not self.stop_evt.is_set():
            try:
                with contextlib.ExitStack() as stack:
                    dm_ctrl = stack.enter_context(Controller(a.channel))
                    my_ctrl = stack.enter_context(Controller(a.channel))
                    rs_ctrl = stack.enter_context(Controller(a.channel))
                    dm1 = dm_ctrl.add_damiao_motor(dm1_id, dm1_fid, a.dm1_model)
                    dm2 = dm_ctrl.add_damiao_motor(dm2_id, dm2_fid, a.dm2_model)
                    mym = my_ctrl.add_myactuator_motor(my_id, my_fid, a.my_model)
                    rsm = rs_ctrl.add_robstride_motor(rs_id, rs_fid, a.rs_model)
                    try:
                        dm_ctrl.enable_all()
                        my_ctrl.enable_all()
                        rs_ctrl.enable_all()
                        dm1.ensure_mode(Mode.POS_VEL, 1000)
                        dm2.ensure_mode(Mode.POS_VEL, 1000)
                        mym.ensure_mode(Mode.POS_VEL, 1000)
                        rsm.ensure_mode(Mode.MIT, 1000)
                        with self.lock:
                            self.state.enabled = True
                            self.state.last_error = ""

                        while not self.stop_evt.is_set():
                            with self.lock:
                                enabled = self.state.enabled
                                dm1_cfg = self.state.motors["dm1"]
                                dm2_cfg = self.state.motors["dm2"]
                                my_cfg = self.state.motors["my"]
                                rs_cfg = self.state.motors["rs"]
                            if enabled:
                                if dm1_cfg.active:
                                    dm1.send_pos_vel(dm1_cfg.pos, dm1_cfg.vlim)
                                if dm2_cfg.active:
                                    dm2.send_pos_vel(dm2_cfg.pos, dm2_cfg.vlim)
                                if my_cfg.active:
                                    mym.send_pos_vel(my_cfg.pos, my_cfg.vlim)
                                if rs_cfg.active:
                                    rs_pos = rs_cfg.pos * rs_cfg.dir_sign
                                    rsm.send_mit(rs_pos, rs_cfg.vel, rs_cfg.kp, rs_cfg.kd, rs_cfg.tau)
                            time.sleep(max(1, a.dt_ms) / 1000.0)
                    finally:
                        for m in (dm1, dm2, mym, rsm):
                            try:
                                m.close()
                            except Exception:
                                pass
            except Exception as e:
                self._set_error(str(e))
                time.sleep(0.5)

    def close(self) -> None:
        self.stop_evt.set()
        self.thread.join(timeout=2.0)


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="WS bridge demo based on Python binding controllers")
    p.add_argument("--bind", default="127.0.0.1")
    p.add_argument("--port", type=int, default=9010)
    p.add_argument("--channel", default="can0")
    p.add_argument("--dt-ms", type=int, default=20)

    p.add_argument("--dm1-model", default="4340P")
    p.add_argument("--dm1-id", default="0x01")
    p.add_argument("--dm1-fid", default="0x11")
    p.add_argument("--dm2-model", default="4310")
    p.add_argument("--dm2-id", default="0x07")
    p.add_argument("--dm2-fid", default="0x17")

    p.add_argument("--my-model", default="X8")
    p.add_argument("--my-id", default="1")
    p.add_argument("--my-fid", default="0x241")

    p.add_argument("--rs-model", default="rs-00")
    p.add_argument("--rs-id", default="127")
    p.add_argument("--rs-fid", default="0xFE")
    return p.parse_args()


async def ws_main(args: argparse.Namespace) -> None:
    import websockets

    bridge = Bridge(args)
    print(f"[binding-ws] ws://{args.bind}:{args.port} channel={args.channel} dt_ms={args.dt_ms}")
    print("[binding-ws] motors: dm1, dm2, my, rs")

    async def handler(ws) -> None:
        await ws.send(json.dumps({"type": "hello", "state": bridge._snapshot()}, ensure_ascii=False))
        async for text in ws:
            try:
                msg = json.loads(text)
            except Exception as e:
                await ws.send(json.dumps({"ok": False, "error": f"invalid json: {e}"}, ensure_ascii=False))
                continue
            resp = bridge.apply_message(msg)
            await ws.send(json.dumps(resp, ensure_ascii=False))

    try:
        async with websockets.serve(handler, args.bind, args.port):
            await asyncio.Future()
    finally:
        bridge.close()


def main() -> None:
    args = parse_args()
    try:
        asyncio.run(ws_main(args))
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
