#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time

from motorbridge import Controller, Mode


def _parse_id(text: str) -> int:
    return int(text, 0)


def _parse_id_list(text: str) -> list[int]:
    if not text.strip():
        return []
    return [int(x.strip(), 0) for x in text.replace(" ", ",").split(",") if x.strip()]


def _default_feedback_id(mid: int) -> int:
    return 0x10 + (mid & 0x0F)


def main() -> None:
    p = argparse.ArgumentParser(
        description=(
            "Multi-joint dm-serial teleoperation demo: read leader motor states from one serial "
            "bridge, mirror target positions to follower motors on another serial bridge."
        )
    )
    p.add_argument("--leader-serial-port", default="/dev/ttyACM0")
    p.add_argument("--follower-serial-port", default="/dev/ttyACM1")
    p.add_argument("--leader-serial-baud", type=int, default=921600)
    p.add_argument("--follower-serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4310")
    # Single-joint args kept for backward compatibility.
    p.add_argument("--leader-id", default="0x07")
    p.add_argument("--leader-feedback-id", default="")
    p.add_argument("--follower-id", default="")
    p.add_argument("--follower-feedback-id", default="")
    # Multi-joint args (comma-separated), e.g. "0x01,0x04,0x05,0x06,0x07"
    p.add_argument("--leader-ids", default="")
    p.add_argument("--leader-feedback-ids", default="")
    p.add_argument("--follower-ids", default="")
    p.add_argument("--follower-feedback-ids", default="")
    p.add_argument(
        "--leader-mode",
        default="mit-zero",
        choices=["mit-zero", "disable-readonly"],
        help="leader behavior: mit-zero (enabled MIT with zero torque) or disable-readonly",
    )
    p.add_argument("--mode", default="pos-vel", choices=["pos-vel", "mit"])
    p.add_argument("--vlim", type=float, default=2.0, help="follower velocity limit for pos-vel")
    p.add_argument("--kp", type=float, default=20.0, help="stiffness for mit mode")
    p.add_argument("--kd", type=float, default=1.0, help="damping for mit mode")
    p.add_argument("--tau", type=float, default=0.0, help="feedforward torque for mit mode")
    p.add_argument("--pos-scale", type=float, default=1.0)
    p.add_argument("--pos-offset", type=float, default=0.0)
    p.add_argument(
        "--feedback-wait-ms",
        type=int,
        default=3,
        help="wait after batch request_feedback before polling (improves multi-joint dm-serial stability)",
    )
    p.add_argument("--loop", type=int, default=10000)
    p.add_argument("--dt-ms", type=int, default=20)
    p.add_argument("--ensure-timeout-ms", type=int, default=1000)
    args = p.parse_args()

    leader_ids = _parse_id_list(args.leader_ids) or [_parse_id(args.leader_id)]
    follower_ids = _parse_id_list(args.follower_ids)
    if not follower_ids:
        if args.follower_id.strip():
            follower_ids = [_parse_id(args.follower_id)]
        else:
            follower_ids = list(leader_ids)

    leader_fids = _parse_id_list(args.leader_feedback_ids)
    if not leader_fids:
        if args.leader_feedback_id.strip():
            leader_fids = [_parse_id(args.leader_feedback_id)]
        else:
            leader_fids = [_default_feedback_id(mid) for mid in leader_ids]

    follower_fids = _parse_id_list(args.follower_feedback_ids)
    if not follower_fids:
        if args.follower_feedback_id.strip():
            follower_fids = [_parse_id(args.follower_feedback_id)]
        else:
            follower_fids = [_default_feedback_id(mid) for mid in follower_ids]

    if len(leader_ids) != len(follower_ids):
        raise ValueError("leader_ids and follower_ids length mismatch")
    if len(leader_fids) != len(leader_ids):
        if len(leader_fids) == 1 and len(leader_ids) > 1:
            leader_fids = leader_fids * len(leader_ids)
        else:
            raise ValueError("leader_feedback_ids length mismatch")
    if len(follower_fids) != len(follower_ids):
        if len(follower_fids) == 1 and len(follower_ids) > 1:
            follower_fids = follower_fids * len(follower_ids)
        else:
            raise ValueError("follower_feedback_ids length mismatch")

    joints = list(zip(leader_ids, leader_fids, follower_ids, follower_fids))
    dt_s = max(0, int(args.dt_ms)) / 1000.0
    fb_wait_s = max(0, int(args.feedback_wait_ms)) / 1000.0
    loop_n = max(1, int(args.loop))
    leader_mode = str(args.leader_mode)

    print(
        f"demo=dm-serial-tele-multi leader={args.leader_serial_port}@{args.leader_serial_baud} "
        f"follower={args.follower_serial_port}@{args.follower_serial_baud} "
        f"joints={len(joints)} mode={args.mode} model={args.model} "
        f"dt_ms={args.dt_ms} leader_mode={leader_mode}"
    )

    with (
        Controller.from_dm_serial(args.leader_serial_port, args.leader_serial_baud) as leader_ctrl,
        Controller.from_dm_serial(args.follower_serial_port, args.follower_serial_baud) as follower_ctrl,
    ):
        leaders = [leader_ctrl.add_damiao_motor(lid, lfid, args.model) for lid, lfid, _, _ in joints]
        followers = [
            follower_ctrl.add_damiao_motor(fid, ffid, args.model) for _, _, fid, ffid in joints
        ]
        try:
            if leader_mode == "disable-readonly":
                for leader in leaders:
                    try:
                        leader.disable()
                    except Exception:
                        pass

            for follower in followers:
                try:
                    follower.disable()
                except Exception:
                    pass

            follower_ctrl.enable_all()
            if args.mode == "pos-vel":
                for follower in followers:
                    follower.ensure_mode(Mode.POS_VEL, int(args.ensure_timeout_ms))
            else:
                for follower in followers:
                    follower.ensure_mode(Mode.MIT, int(args.ensure_timeout_ms))

            if leader_mode == "mit-zero":
                leader_ctrl.enable_all()
                for leader in leaders:
                    leader.ensure_mode(Mode.MIT, int(args.ensure_timeout_ms))

            missed = [0 for _ in joints]
            for i in range(loop_n):
                if leader_mode == "mit-zero":
                    for leader in leaders:
                        leader.send_mit(0.0, 0.0, 0.0, 0.0, 0.0)

                # 1) batch request leader feedback first (avoid per-joint immediate poll race)
                for leader in leaders:
                    try:
                        leader.request_feedback()
                    except Exception:
                        pass

                # 2) give bridge/motor a short window to return frames, then poll once to drain queue
                if fb_wait_s > 0:
                    time.sleep(fb_wait_s)
                try:
                    leader_ctrl.poll_feedback_once()
                except Exception:
                    pass

                # 3) use refreshed cached states to drive follower joints
                row = []
                for idx, (leader, follower) in enumerate(zip(leaders, followers)):
                    st = leader.get_state()
                    lid, _, fid, _ = joints[idx]
                    if st is None:
                        missed[idx] += 1
                        row.append(f"L0x{lid:X}->F0x{fid:X}:no_fb({missed[idx]})")
                        continue

                    target_pos = float(st.pos) * float(args.pos_scale) + float(args.pos_offset)
                    if args.mode == "pos-vel":
                        follower.send_pos_vel(target_pos, float(args.vlim))
                    else:
                        follower.send_mit(
                            target_pos, 0.0, float(args.kp), float(args.kd), float(args.tau)
                        )
                    row.append(f"L0x{lid:X}:{st.pos:+.3f}->F0x{fid:X}:{target_pos:+.3f}")

                print(f"#{i+1}/{loop_n} " + " | ".join(row))

                if dt_s > 0:
                    time.sleep(dt_s)
        finally:
            for follower in followers:
                try:
                    follower.disable()
                except Exception:
                    pass
            for leader in leaders:
                try:
                    leader.disable()
                except Exception:
                    pass
            for follower in followers:
                follower.close()
            for leader in leaders:
                leader.close()


if __name__ == "__main__":
    main()
