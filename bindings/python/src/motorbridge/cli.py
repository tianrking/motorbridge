from __future__ import annotations

import argparse
import time

from .core import Controller
from .models import Mode


def _mode_to_enum(mode: str) -> Mode:
    return {
        "mit": Mode.MIT,
        "pos-vel": Mode.POS_VEL,
        "vel": Mode.VEL,
        "force-pos": Mode.FORCE_POS,
    }[mode]


def _parse_id(text: str) -> int:
    return int(text, 0)


def _parse_rids(text: str) -> list[int]:
    return [int(x.strip(), 0) for x in text.split(",") if x.strip()]


def _add_common_args(p: argparse.ArgumentParser) -> None:
    p.add_argument("--channel", default="can0")
    p.add_argument("--model", default="4340")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="motorbridge Python SDK CLI")
    sub = p.add_subparsers(dest="command")

    run = sub.add_parser("run", help="send control commands (default command)")
    _add_common_args(run)
    run.add_argument(
        "--mode",
        default="mit",
        choices=["enable", "disable", "mit", "pos-vel", "vel", "force-pos"],
    )
    run.add_argument("--loop", type=int, default=100)
    run.add_argument("--dt-ms", type=int, default=20)
    run.add_argument("--ensure-mode", type=int, default=1)
    run.add_argument("--ensure-strict", type=int, default=0)
    run.add_argument("--ensure-timeout-ms", type=int, default=1000)
    run.add_argument("--print-state", type=int, default=1)
    run.add_argument("--pos", type=float, default=0.0)
    run.add_argument("--vel", type=float, default=0.0)
    run.add_argument("--kp", type=float, default=30.0)
    run.add_argument("--kd", type=float, default=1.0)
    run.add_argument("--tau", type=float, default=0.0)
    run.add_argument("--vlim", type=float, default=1.0)
    run.add_argument("--ratio", type=float, default=0.3)

    dump = sub.add_parser("id-dump", help="read key ID/mode/timeout registers")
    _add_common_args(dump)
    dump.add_argument("--timeout-ms", type=int, default=500)
    dump.add_argument("--rids", default="7,8,9,10,21,22,23")

    set_id = sub.add_parser("id-set", help="write ESC_ID/MST_ID and optionally store")
    _add_common_args(set_id)
    set_id.add_argument("--new-motor-id", default="")
    set_id.add_argument("--new-feedback-id", default="")
    set_id.add_argument("--store", type=int, default=1)
    set_id.add_argument("--verify", type=int, default=1)
    set_id.add_argument("--timeout-ms", type=int, default=800)

    scan = sub.add_parser("scan", help="scan active motor IDs by register probing")
    scan.add_argument("--channel", default="can0")
    scan.add_argument("--model", default="4340")
    scan.add_argument("--start-id", default="0x01")
    scan.add_argument("--end-id", default="0x10")
    scan.add_argument("--feedback-base", default="0x10")
    scan.add_argument("--timeout-ms", type=int, default=80)

    return p


def _parse_with_legacy_support() -> argparse.Namespace:
    parser = _build_parser()
    args, extras = parser.parse_known_args()
    if args.command is not None:
        if extras:
            parser.error(f"unrecognized arguments: {' '.join(extras)}")
        return args

    legacy = argparse.ArgumentParser(description="motorbridge Python SDK CLI (legacy run mode)")
    _add_common_args(legacy)
    legacy.add_argument(
        "--mode",
        default="mit",
        choices=["enable", "disable", "mit", "pos-vel", "vel", "force-pos"],
    )
    legacy.add_argument("--loop", type=int, default=100)
    legacy.add_argument("--dt-ms", type=int, default=20)
    legacy.add_argument("--ensure-mode", type=int, default=1)
    legacy.add_argument("--ensure-strict", type=int, default=0)
    legacy.add_argument("--ensure-timeout-ms", type=int, default=1000)
    legacy.add_argument("--print-state", type=int, default=1)
    legacy.add_argument("--pos", type=float, default=0.0)
    legacy.add_argument("--vel", type=float, default=0.0)
    legacy.add_argument("--kp", type=float, default=30.0)
    legacy.add_argument("--kd", type=float, default=1.0)
    legacy.add_argument("--tau", type=float, default=0.0)
    legacy.add_argument("--vlim", type=float, default=1.0)
    legacy.add_argument("--ratio", type=float, default=0.3)
    legacy_args = legacy.parse_args()
    legacy_args.command = "run"
    return legacy_args


def _run_command(args: argparse.Namespace) -> None:
    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    print(
        f"command=run channel={args.channel} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} mode={args.mode}"
    )

    with Controller(args.channel) as ctrl:
        motor = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
        try:
            if args.mode not in ("enable", "disable"):
                ctrl.enable_all()
                time.sleep(0.3)

            if args.ensure_mode and args.mode not in ("enable", "disable"):
                try:
                    motor.ensure_mode(_mode_to_enum(args.mode), args.ensure_timeout_ms)
                except Exception as e:
                    if args.ensure_strict:
                        raise
                    print(f"[warn] ensure_mode failed: {e}; continue anyway")

            for i in range(args.loop):
                if args.mode == "enable":
                    motor.enable()
                    motor.request_feedback()
                elif args.mode == "disable":
                    motor.disable()
                    motor.request_feedback()
                elif args.mode == "mit":
                    motor.send_mit(args.pos, args.vel, args.kp, args.kd, args.tau)
                elif args.mode == "pos-vel":
                    motor.send_pos_vel(args.pos, args.vlim)
                elif args.mode == "vel":
                    motor.send_vel(args.vel)
                elif args.mode == "force-pos":
                    motor.send_force_pos(args.pos, args.vlim, args.ratio)

                if args.print_state:
                    st = motor.get_state()
                    if st is None:
                        print(f"#{i} no feedback yet")
                    else:
                        print(
                            f"#{i} pos={st.pos:+.3f} vel={st.vel:+.3f} "
                            f"torq={st.torq:+.3f} status={st.status_code}"
                        )
                time.sleep(max(args.dt_ms, 0) / 1000.0)
        finally:
            motor.close()


def _id_dump_command(args: argparse.Namespace) -> None:
    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    rids = _parse_rids(args.rids)
    print(
        f"command=id-dump channel={args.channel} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X}"
    )
    ctrl = Controller(args.channel)
    motor = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
    try:
        for rid in rids:
            try:
                value = motor.get_register_u32(rid, args.timeout_ms)
                print(f"rid={rid:>3} (u32) = {value} (0x{value:X})")
            except Exception as e_u32:
                try:
                    value_f = motor.get_register_f32(rid, args.timeout_ms)
                    print(f"rid={rid:>3} (f32) = {value_f:.6f}")
                except Exception:
                    print(f"rid={rid:>3} read failed: {e_u32}")
    finally:
        motor.close()
        ctrl.close_bus()
        ctrl.close()


def _id_set_command(args: argparse.Namespace) -> None:
    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    new_motor_id = _parse_id(args.new_motor_id) if args.new_motor_id else motor_id
    new_feedback_id = _parse_id(args.new_feedback_id) if args.new_feedback_id else feedback_id
    print(
        f"command=id-set channel={args.channel} model={args.model} "
        f"old_motor_id=0x{motor_id:X} old_feedback_id=0x{feedback_id:X} "
        f"new_motor_id=0x{new_motor_id:X} new_feedback_id=0x{new_feedback_id:X}"
    )

    ctrl = Controller(args.channel)
    motor = ctrl.add_damiao_motor(motor_id, feedback_id, args.model)
    try:
        if new_feedback_id != feedback_id:
            motor.write_register_u32(7, new_feedback_id)
            print(f"write rid=7 (MST_ID) <= 0x{new_feedback_id:X}")
        if new_motor_id != motor_id:
            motor.write_register_u32(8, new_motor_id)
            print(f"write rid=8 (ESC_ID) <= 0x{new_motor_id:X}")
        if args.store:
            motor.store_parameters()
            print("store_parameters sent")
    finally:
        motor.close()
        ctrl.close_bus()
        ctrl.close()

    if not args.verify:
        return

    verify_ctrl = Controller(args.channel)
    verify_motor = verify_ctrl.add_damiao_motor(new_motor_id, new_feedback_id, args.model)
    try:
        esc = verify_motor.get_register_u32(8, args.timeout_ms)
        mst = verify_motor.get_register_u32(7, args.timeout_ms)
        print(f"verify rid=8 (ESC_ID): 0x{esc:X}")
        print(f"verify rid=7 (MST_ID): 0x{mst:X}")
        if esc != new_motor_id or mst != new_feedback_id:
            raise RuntimeError(
                f"verify failed: expected ESC_ID=0x{new_motor_id:X}, MST_ID=0x{new_feedback_id:X}, "
                f"got ESC_ID=0x{esc:X}, MST_ID=0x{mst:X}"
            )
        print("verify ok")
    finally:
        verify_motor.close()
        verify_ctrl.close_bus()
        verify_ctrl.close()


def _scan_command(args: argparse.Namespace) -> None:
    start_id = _parse_id(args.start_id)
    end_id = _parse_id(args.end_id)
    feedback_base = _parse_id(args.feedback_base)
    if end_id < start_id:
        raise ValueError("end-id must be >= start-id")
    print(
        f"command=scan channel={args.channel} model={args.model} "
        f"id_range=[0x{start_id:X},0x{end_id:X}] timeout_ms={args.timeout_ms}"
    )
    found: list[tuple[int, int, int]] = []
    for mid in range(start_id, end_id + 1):
        fid = feedback_base + (mid & 0x0F)
        ctrl = Controller(args.channel)
        try:
            motor = ctrl.add_damiao_motor(mid, fid, args.model)
            try:
                esc_id = motor.get_register_u32(8, args.timeout_ms)
                mst_id = motor.get_register_u32(7, args.timeout_ms)
                found.append((mid, esc_id, mst_id))
                print(f"[hit] probe=0x{mid:02X} esc_id=0x{esc_id:X} mst_id=0x{mst_id:X}")
            except Exception:
                print(f"[.. ] probe=0x{mid:02X} no reply")
            finally:
                motor.close()
        finally:
            ctrl.close_bus()
            ctrl.close()

    print(f"scan done: {len(found)} motor(s) found")
    for probe, esc, mst in found:
        print(f"  probe=0x{probe:02X} ESC_ID=0x{esc:X} MST_ID=0x{mst:X}")


def main() -> None:
    args = _parse_with_legacy_support()
    if args.command == "run":
        _run_command(args)
    elif args.command == "id-dump":
        _id_dump_command(args)
    elif args.command == "id-set":
        _id_set_command(args)
    elif args.command == "scan":
        _scan_command(args)
    else:
        raise RuntimeError(f"unknown command: {args.command}")


if __name__ == "__main__":
    main()
