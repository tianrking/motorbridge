#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def main() -> None:
    p = argparse.ArgumentParser(
        description=(
            "Single-joint dm-serial teleoperation demo: leader on one serial bridge, "
            "follower on another serial bridge."
        )
    )
    p.add_argument("--leader-serial-port", default="/dev/ttyACM0")
    p.add_argument("--follower-serial-port", default="/dev/ttyACM1")
    p.add_argument("--leader-serial-baud", type=int, default=921600)
    p.add_argument("--follower-serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    p.add_argument("--leader-id", default="0x07")
    p.add_argument("--leader-feedback-id", default="0x17")
    p.add_argument("--follower-id", default="0x07")
    p.add_argument("--follower-feedback-id", default="0x17")
    p.add_argument(
        "--leader-mode",
        default="mit-zero",
        choices=["mit-zero", "disable-readonly"],
        help="leader behavior: mit-zero (enabled MIT with zero torque) or disable-readonly",
    )
    p.add_argument("--mode", default="pos-vel", choices=["pos-vel", "mit"])
    p.add_argument("--vlim", type=float, default=2.0)
    p.add_argument("--kp", type=float, default=20.0)
    p.add_argument("--kd", type=float, default=1.0)
    p.add_argument("--tau", type=float, default=0.0)
    p.add_argument("--pos-scale", type=float, default=1.0)
    p.add_argument("--pos-offset", type=float, default=0.0)
    p.add_argument("--feedback-wait-ms", type=int, default=3)
    p.add_argument("--loop", type=int, default=10000)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--ensure-timeout-ms", type=int, default=1000)
    args = p.parse_args()

    leader_id = _parse_id(args.leader_id)
    leader_fid = _parse_id(args.leader_feedback_id)
    follower_id = _parse_id(args.follower_id)
    follower_fid = _parse_id(args.follower_feedback_id)
    dt_s = max(0, int(args.dt_ms)) / 1000.0
    fb_wait_s = max(0, int(args.feedback_wait_ms)) / 1000.0
    loop_n = max(1, int(args.loop))
    leader_mode = str(args.leader_mode)

    print(
        f"demo=dm-serial-tele-single leader={args.leader_serial_port}@{args.leader_serial_baud}"
        f"(id=0x{leader_id:X},fid=0x{leader_fid:X}) "
        f"follower={args.follower_serial_port}@{args.follower_serial_baud}"
        f"(id=0x{follower_id:X},fid=0x{follower_fid:X}) "
        f"mode={args.mode} dt_ms={args.dt_ms} leader_mode={leader_mode}"
    )

    with (
        Controller.from_dm_serial(args.leader_serial_port, args.leader_serial_baud) as leader_ctrl,
        Controller.from_dm_serial(args.follower_serial_port, args.follower_serial_baud) as follower_ctrl,
    ):
        leader = leader_ctrl.add_damiao_motor(leader_id, leader_fid, args.model)
        follower = follower_ctrl.add_damiao_motor(follower_id, follower_fid, args.model)
        try:
            if leader_mode == "disable-readonly":
                try:
                    leader.disable()
                except Exception:
                    pass

            try:
                follower.disable()
            except Exception:
                pass

            follower_ctrl.enable_all()
            if args.mode == "pos-vel":
                follower.ensure_mode(Mode.POS_VEL, int(args.ensure_timeout_ms))
            else:
                follower.ensure_mode(Mode.MIT, int(args.ensure_timeout_ms))

            if leader_mode == "mit-zero":
                leader_ctrl.enable_all()
                leader.ensure_mode(Mode.MIT, int(args.ensure_timeout_ms))

            missed = 0
            for i in range(loop_n):
                if leader_mode == "mit-zero":
                    leader.send_mit(0.0, 0.0, 0.0, 0.0, 0.0)

                try:
                    leader.request_feedback()
                except Exception:
                    pass
                if fb_wait_s > 0:
                    time.sleep(fb_wait_s)
                try:
                    leader_ctrl.poll_feedback_once()
                except Exception:
                    pass

                st = leader.get_state()
                if st is None:
                    missed += 1
                    print(f"#{i+1}/{loop_n} leader=no_feedback missed={missed}")
                    if dt_s > 0:
                        time.sleep(dt_s)
                    continue

                target_pos = float(st.pos) * float(args.pos_scale) + float(args.pos_offset)
                if args.mode == "pos-vel":
                    follower.send_pos_vel(target_pos, float(args.vlim))
                else:
                    follower.send_mit(target_pos, 0.0, float(args.kp), float(args.kd), float(args.tau))

                print(
                    f"#{i+1}/{loop_n} leader_pos={st.pos:+.3f} leader_vel={st.vel:+.3f} "
                    f"target_pos={target_pos:+.3f}"
                )
                if dt_s > 0:
                    time.sleep(dt_s)
        finally:
            try:
                follower.disable()
            except Exception:
                pass
            try:
                leader.disable()
            except Exception:
                pass
            follower.close()
            leader.close()


if __name__ == "__main__":
    main()
