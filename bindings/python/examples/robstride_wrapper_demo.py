#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    parser = argparse.ArgumentParser(description="RobStride wrapper demo for Python SDK")
    parser.add_argument("--channel", default="can0")
    parser.add_argument("--model", default="rs-00")
    parser.add_argument("--motor-id", default="127")
    parser.add_argument("--feedback-id", default="0xFF")
    parser.add_argument("--mode", choices=["ping", "read-param", "mit", "vel"], default="ping")
    parser.add_argument("--pos", type=float, default=0.0)
    parser.add_argument("--vel", type=float, default=0.0)
    parser.add_argument("--kp", type=float, default=8.0)
    parser.add_argument("--kd", type=float, default=0.2)
    parser.add_argument("--tau", type=float, default=0.0)
    parser.add_argument("--loop", type=int, default=20)
    parser.add_argument("--dt-ms", type=int, default=50)
    parser.add_argument("--param-id", default="0x7019")
    parser.add_argument("--param-timeout-ms", type=int, default=1000)
    args = parser.parse_args()

    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    param_id = _parse_id(args.param_id)

    with Controller(args.channel) as ctrl:
        motor = ctrl.add_robstride_motor(motor_id, feedback_id, args.model)
        try:
            if args.mode == "ping":
                device_id, responder_id = motor.robstride_ping()
                print(f"ping ok device_id={device_id} responder_id={responder_id}")
                print(motor.get_state())
                return

            if args.mode == "read-param":
                value = motor.robstride_get_param_f32(param_id, args.param_timeout_ms)
                print(f"param 0x{param_id:04X} = {value}")
                print(motor.get_state())
                return

            ctrl.enable_all()
            motor.ensure_mode(Mode.MIT if args.mode == "mit" else Mode.VEL, 1000)

            for i in range(args.loop):
                if args.mode == "mit":
                    motor.send_mit(args.pos, args.vel, args.kp, args.kd, args.tau)
                else:
                    motor.send_vel(args.vel)
                print(f"#{i} {motor.get_state()}")
                if args.dt_ms > 0:
                    time.sleep(args.dt_ms / 1000.0)
        finally:
            motor.close()


if __name__ == "__main__":
    main()
