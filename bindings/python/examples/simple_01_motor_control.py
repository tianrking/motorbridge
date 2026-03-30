#!/usr/bin/env python3
"""Simple single-motor control template (beginner-friendly).

This is the easiest example to start with:
1) open one controller on one CAN channel,
2) add exactly one motor,
3) enable + ensure mode,
4) send one command type in a fixed-rate loop.

By default it runs Damiao in POS_VEL mode.
Other vendor snippets are included as commented lines below.

Beginner checklist:
- make sure only one program is sending to `can0`,
- start with `--dt-ms 20` (or 30/50 if bus is busy),
- verify the motor moves first, then tune `--pos` / `--vlim`.
"""
import argparse
import time

from motorbridge import Controller, Mode
from motorbridge.errors import CallError


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Simple single motor demo")
    p.add_argument(
        "--channel",
        default="can0",
        help="SocketCAN interface name (e.g. can0, can1, slcan0). Default: can0",
    )
    p.add_argument(
        "--loop",
        type=int,
        default=120,
        help="How many control iterations to send. Default: 120",
    )
    p.add_argument(
        "--dt-ms",
        type=int,
        default=20,
        help=(
            "Control period in milliseconds. Keep >=20ms on busy bus. "
            "If you hit os error 105, try 30 or 50."
        ),
    )
    p.add_argument(
        "--pos",
        type=float,
        default=1.0,
        help="Target position (radians) used by send_pos_vel. Default: 1.0",
    )
    p.add_argument(
        "--vlim",
        type=float,
        default=1.0,
        help="Speed limit for POS_VEL command. Default: 1.0",
    )
    return p.parse_args()


args = parse_args()
dt_s = max(args.dt_ms, 1) / 1000.0

# Create one controller and auto-close it with context manager.
# One controller = one vendor family in this SDK.
with Controller(args.channel) as ctrl:
    # 1) Default active choice: Damiao.
    # add_damiao_motor(motor_id, feedback_id, model)
    # - motor_id: command target ID on CAN
    # - feedback_id: expected feedback frame ID
    # - model: motor model string
    motor = ctrl.add_damiao_motor(0x01, 0x11, "4340P")
    target_mode = Mode.POS_VEL

    # 2) Other vendor templates (commented by default; uncomment one block to switch)
    # MyActuator:
    # Typical: id=1, feedback=0x241, model=X8
    # motor = ctrl.add_myactuator_motor(1, 0x241, "X8")
    # target_mode = Mode.POS_VEL

    # RobStride:
    # Typical: id=127, feedback=0xFE, model=rs-00
    # motor = ctrl.add_robstride_motor(127, 0xFE, "rs-00")
    # target_mode = Mode.MIT

    # HighTorque:
    # Typical: id=1, feedback=0x01, model=hightorque
    # motor = ctrl.add_hightorque_motor(1, 0x01, "hightorque")
    # target_mode = Mode.MIT

    # Enable motor output and make sure control mode is ready.
    # If ensure_mode times out, check CAN wiring, IDs, and that no other sender is flooding bus.
    ctrl.enable_all()
    motor.ensure_mode(target_mode, 1000)

    # Main control loop:
    # - Damiao/MyActuator: keep sending POS_VEL target.
    # - RobStride/HighTorque: comment send_pos_vel and use send_mit line.
    # Why "keep sending"? Most drivers are streaming controllers, not one-shot latches.
    for i in range(args.loop):
        t0 = time.time()
        try:
            motor.send_pos_vel(args.pos, args.vlim)  # pos, vlim
            # motor.send_mit(args.pos, 0.0, 2.0, 1.0, 0.0)  # pos, vel, kp, kd, tau
        except CallError as e:
            print(f"#{i} send error: {e}")
            break
        print(f"#{i} state={motor.get_state()}")
        # Keep fixed loop period to prevent CAN TX saturation.
        elapsed = time.time() - t0
        if elapsed < dt_s:
            time.sleep(dt_s - elapsed)
