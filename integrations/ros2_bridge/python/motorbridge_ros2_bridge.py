#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import time
from typing import Any

import rclpy
from rclpy.node import Node
from std_msgs.msg import String

from motorbridge import Controller, Mode


class MotorBridgeRos2Bridge(Node):
    def __init__(self, channel: str, model: str, motor_id: int, feedback_id: int, dt_ms: int) -> None:
        super().__init__("motorbridge_ros2_bridge")
        self.channel = channel
        self.model = model
        self.motor_id = motor_id
        self.feedback_id = feedback_id
        self.dt_ms = max(dt_ms, 1)

        self.ctrl = Controller(channel)
        self.motor = self.ctrl.add_damiao_motor(motor_id, feedback_id, model)

        self.cmd_sub = self.create_subscription(String, "/motorbridge/cmd", self._on_cmd, 10)
        self.state_pub = self.create_publisher(String, "/motorbridge/state", 10)
        self.event_pub = self.create_publisher(String, "/motorbridge/event", 10)

        self.active_control: dict[str, Any] | None = None
        self.timer = self.create_timer(self.dt_ms / 1000.0, self._on_tick)

        self._emit_event("bridge_started", {
            "channel": self.channel,
            "model": self.model,
            "motor_id": self.motor_id,
            "feedback_id": self.feedback_id,
            "dt_ms": self.dt_ms,
        })

    def destroy_node(self) -> bool:
        try:
            self.motor.close()
        except Exception:
            pass
        try:
            self.ctrl.close_bus()
        except Exception:
            pass
        try:
            self.ctrl.close()
        except Exception:
            pass
        return super().destroy_node()

    def _emit_event(self, kind: str, payload: dict[str, Any]) -> None:
        msg = String()
        msg.data = json.dumps({"event": kind, "payload": payload}, ensure_ascii=True)
        self.event_pub.publish(msg)

    def _publish_state(self) -> None:
        st = self.motor.get_state()
        payload: dict[str, Any]
        if st is None:
            payload = {"has_value": False}
        else:
            payload = {
                "has_value": True,
                "can_id": st.can_id,
                "arbitration_id": st.arbitration_id,
                "status_code": st.status_code,
                "pos": st.pos,
                "vel": st.vel,
                "torq": st.torq,
                "t_mos": st.t_mos,
                "t_rotor": st.t_rotor,
            }
        msg = String()
        msg.data = json.dumps(payload, ensure_ascii=True)
        self.state_pub.publish(msg)

    def _apply_control(self, cmd: dict[str, Any]) -> None:
        op = str(cmd.get("op", "")).lower()
        if op == "mit":
            self.motor.send_mit(
                float(cmd.get("pos", 0.0)),
                float(cmd.get("vel", 0.0)),
                float(cmd.get("kp", 30.0)),
                float(cmd.get("kd", 1.0)),
                float(cmd.get("tau", 0.0)),
            )
        elif op in ("pos_vel", "pos-vel"):
            self.motor.send_pos_vel(float(cmd.get("pos", 0.0)), float(cmd.get("vlim", 1.0)))
        elif op == "vel":
            self.motor.send_vel(float(cmd.get("vel", 0.0)))
        elif op in ("force_pos", "force-pos"):
            self.motor.send_force_pos(
                float(cmd.get("pos", 0.0)),
                float(cmd.get("vlim", 1.0)),
                float(cmd.get("ratio", 0.3)),
            )
        else:
            raise ValueError(f"unsupported control op: {op}")

    def _ensure_mode_for(self, op: str, timeout_ms: int) -> None:
        if op == "mit":
            self.motor.ensure_mode(Mode.MIT, timeout_ms)
        elif op in ("pos_vel", "pos-vel"):
            self.motor.ensure_mode(Mode.POS_VEL, timeout_ms)
        elif op == "vel":
            self.motor.ensure_mode(Mode.VEL, timeout_ms)
        elif op in ("force_pos", "force-pos"):
            self.motor.ensure_mode(Mode.FORCE_POS, timeout_ms)

    def _scan(self, cmd: dict[str, Any]) -> None:
        start_id = int(cmd.get("start_id", 1))
        end_id = int(cmd.get("end_id", 16))
        feedback_base = int(cmd.get("feedback_base", 16))
        timeout_ms = int(cmd.get("timeout_ms", 100))

        if end_id < start_id:
            raise ValueError("end_id must be >= start_id")

        hits = []
        for mid in range(start_id, end_id + 1):
            fid = feedback_base + (mid & 0x0F)
            probe_ctrl = Controller(self.channel)
            try:
                probe_motor = probe_ctrl.add_damiao_motor(mid, fid, self.model)
                try:
                    esc = probe_motor.get_register_u32(8, timeout_ms)
                    mst = probe_motor.get_register_u32(7, timeout_ms)
                    hits.append({"probe": mid, "esc_id": esc, "mst_id": mst})
                except Exception:
                    pass
                finally:
                    probe_motor.close()
            finally:
                try:
                    probe_ctrl.close_bus()
                except Exception:
                    pass
                probe_ctrl.close()

        self._emit_event("scan_result", {
            "count": len(hits),
            "hits": hits,
            "start_id": start_id,
            "end_id": end_id,
        })

    def _set_id(self, cmd: dict[str, Any]) -> None:
        old_mid = int(cmd.get("old_motor_id", self.motor_id))
        old_fid = int(cmd.get("old_feedback_id", self.feedback_id))
        new_mid = int(cmd.get("new_motor_id", old_mid))
        new_fid = int(cmd.get("new_feedback_id", old_fid))
        store = bool(cmd.get("store", True))
        verify = bool(cmd.get("verify", True))
        timeout_ms = int(cmd.get("timeout_ms", 1000))

        set_ctrl = Controller(self.channel)
        try:
            set_motor = set_ctrl.add_damiao_motor(old_mid, old_fid, self.model)
            try:
                if new_fid != old_fid:
                    set_motor.write_register_u32(7, new_fid)
                if new_mid != old_mid:
                    set_motor.write_register_u32(8, new_mid)
                if store:
                    set_motor.store_parameters()
            finally:
                set_motor.close()
        finally:
            try:
                set_ctrl.close_bus()
            except Exception:
                pass
            set_ctrl.close()

        payload = {
            "old_motor_id": old_mid,
            "old_feedback_id": old_fid,
            "new_motor_id": new_mid,
            "new_feedback_id": new_fid,
            "store": store,
        }

        if verify:
            time.sleep(0.12)
            verify_ctrl = Controller(self.channel)
            try:
                verify_motor = verify_ctrl.add_damiao_motor(new_mid, new_fid, self.model)
                try:
                    esc = verify_motor.get_register_u32(8, timeout_ms)
                    mst = verify_motor.get_register_u32(7, timeout_ms)
                finally:
                    verify_motor.close()
            finally:
                try:
                    verify_ctrl.close_bus()
                except Exception:
                    pass
                verify_ctrl.close()
            payload["verify"] = {"esc_id": esc, "mst_id": mst, "ok": esc == new_mid and mst == new_fid}

        self._emit_event("set_id_result", payload)

    def _verify(self, cmd: dict[str, Any]) -> None:
        mid = int(cmd.get("motor_id", self.motor_id))
        fid = int(cmd.get("feedback_id", self.feedback_id))
        timeout_ms = int(cmd.get("timeout_ms", 1000))

        verify_ctrl = Controller(self.channel)
        try:
            verify_motor = verify_ctrl.add_damiao_motor(mid, fid, self.model)
            try:
                esc = verify_motor.get_register_u32(8, timeout_ms)
                mst = verify_motor.get_register_u32(7, timeout_ms)
            finally:
                verify_motor.close()
        finally:
            try:
                verify_ctrl.close_bus()
            except Exception:
                pass
            verify_ctrl.close()

        self._emit_event("verify_result", {
            "motor_id": mid,
            "feedback_id": fid,
            "esc_id": esc,
            "mst_id": mst,
            "ok": esc == mid and mst == fid,
        })

    def _on_cmd(self, msg: String) -> None:
        try:
            cmd = json.loads(msg.data)
            if not isinstance(cmd, dict):
                raise ValueError("cmd payload must be a JSON object")

            op = str(cmd.get("op", "")).lower()
            ensure_mode = bool(cmd.get("ensure_mode", True))
            ensure_timeout_ms = int(cmd.get("ensure_timeout_ms", 1000))
            continuous = bool(cmd.get("continuous", False))

            if op == "enable":
                self.ctrl.enable_all()
                self.motor.request_feedback()
                self.active_control = None
                self._emit_event("enable_ok", {})
                return

            if op == "disable":
                self.ctrl.disable_all()
                self.motor.request_feedback()
                self.active_control = None
                self._emit_event("disable_ok", {})
                return

            if op == "scan":
                self._scan(cmd)
                return

            if op == "set_id":
                self._set_id(cmd)
                return

            if op == "verify":
                self._verify(cmd)
                return

            if op in ("mit", "pos_vel", "pos-vel", "vel", "force_pos", "force-pos"):
                if ensure_mode:
                    self._ensure_mode_for(op, ensure_timeout_ms)
                self._apply_control(cmd)
                self.active_control = cmd if continuous else None
                self._emit_event("control_ok", {"op": op, "continuous": continuous})
                return

            raise ValueError(f"unsupported op: {op}")
        except Exception as e:
            self.active_control = None
            self._emit_event("error", {"message": str(e)})

    def _on_tick(self) -> None:
        try:
            if self.active_control is not None:
                self._apply_control(self.active_control)
            self._publish_state()
        except Exception as e:
            self.active_control = None
            self._emit_event("error", {"message": str(e)})


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="motorbridge ROS2 bridge")
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4340P")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")
    p.add_argument("--dt-ms", type=int, default=20)
    return p.parse_args()


def main() -> None:
    args = parse_args()
    motor_id = int(args.motor_id, 0)
    feedback_id = int(args.feedback_id, 0)

    rclpy.init()
    node = MotorBridgeRos2Bridge(
        channel=args.channel,
        model=args.model,
        motor_id=motor_id,
        feedback_id=feedback_id,
        dt_ms=args.dt_ms,
    )
    try:
        rclpy.spin(node)
    finally:
        node.destroy_node()
        rclpy.shutdown()


if __name__ == "__main__":
    main()
