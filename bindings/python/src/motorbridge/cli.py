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
    p.add_argument(
        "--vendor",
        default="damiao",
        choices=["damiao", "myactuator", "robstride", "hightorque", "hexfellow"],
    )
    p.add_argument("--channel", default="can0")
    p.add_argument(
        "--transport",
        default="auto",
        choices=["auto", "socketcan", "socketcanfd", "dm-serial"],
    )
    p.add_argument("--serial-port", default="/dev/ttyACM0")
    p.add_argument("--serial-baud", type=int, default=921600)
    p.add_argument("--model", default="4340")
    p.add_argument("--motor-id", default="0x01")
    p.add_argument("--feedback-id", default="0x11")


def _vendor_defaults(vendor: str, model: str, feedback_id: str) -> tuple[str, str]:
    resolved_model = model
    resolved_feedback = feedback_id
    if vendor == "robstride":
        if resolved_model == "4340":
            resolved_model = "rs-00"
        if resolved_feedback == "0x11":
            resolved_feedback = "0xFF"
    elif vendor == "myactuator":
        if resolved_model == "4340":
            resolved_model = "X8"
        if resolved_feedback == "0x11":
            resolved_feedback = "0x241"
    elif vendor == "hightorque":
        if resolved_model == "4340":
            resolved_model = "hightorque"
        if resolved_feedback == "0x11":
            resolved_feedback = "0x01"
    elif vendor == "hexfellow":
        if resolved_model == "4340":
            resolved_model = "hexfellow"
        if resolved_feedback == "0x11":
            resolved_feedback = "0x00"
    return resolved_model, resolved_feedback


def _add_motor(ctrl: Controller, vendor: str, motor_id: int, feedback_id: int, model: str):
    if vendor == "myactuator":
        return ctrl.add_myactuator_motor(motor_id, feedback_id, model)
    if vendor == "robstride":
        return ctrl.add_robstride_motor(motor_id, feedback_id, model)
    if vendor == "hightorque":
        return ctrl.add_hightorque_motor(motor_id, feedback_id, model)
    if vendor == "hexfellow":
        return ctrl.add_hexfellow_motor(motor_id, feedback_id, model)
    return ctrl.add_damiao_motor(motor_id, feedback_id, model)


def _open_controller(args: argparse.Namespace, vendor: str) -> Controller:
    transport = getattr(args, "transport", "auto")
    if transport == "dm-serial":
        if vendor != "damiao":
            raise ValueError("transport=dm-serial is supported only for --vendor damiao")
        return Controller.from_dm_serial(args.serial_port, int(args.serial_baud))
    if vendor == "hexfellow":
        if transport == "socketcan":
            raise ValueError("vendor=hexfellow requires --transport socketcanfd (or auto)")
        return Controller.from_socketcanfd(args.channel)
    if transport == "socketcanfd":
        return Controller.from_socketcanfd(args.channel)
    return Controller(args.channel)


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="motorbridge Python SDK CLI")
    sub = p.add_subparsers(dest="command")

    run = sub.add_parser("run", help="send control commands (default command)")
    _add_common_args(run)
    run.add_argument(
        "--mode",
        default="mit",
        choices=["enable", "disable", "mit", "pos-vel", "vel", "force-pos", "ping"],
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

    scan = sub.add_parser("scan", help="scan active motor IDs")
    scan.add_argument(
        "--vendor",
        default="damiao",
        choices=["damiao", "myactuator", "robstride", "hightorque", "hexfellow", "all"],
    )
    scan.add_argument("--channel", default="can0")
    scan.add_argument(
        "--transport",
        default="auto",
        choices=["auto", "socketcan", "socketcanfd", "dm-serial"],
    )
    scan.add_argument("--serial-port", default="/dev/ttyACM0")
    scan.add_argument("--serial-baud", type=int, default=921600)
    scan.add_argument("--model", default="4340")
    scan.add_argument("--start-id", default="0x01")
    scan.add_argument("--end-id", default="0x10")
    scan.add_argument("--feedback-ids", default="0xFF,0xFE,0x00,0xAA")
    scan.add_argument("--feedback-base", default="0x10")
    scan.add_argument("--timeout-ms", type=int, default=80)
    scan.add_argument("--param-id", default="0x7019")
    scan.add_argument("--param-timeout-ms", type=int, default=120)

    rs_read = sub.add_parser("robstride-read-param", help="read a RobStride parameter")
    _add_common_args(rs_read)
    rs_read.set_defaults(vendor="robstride")
    rs_read.set_defaults(model="rs-00", feedback_id="0xFF")
    rs_read.add_argument("--param-id", required=True)
    rs_read.add_argument("--type", required=True, choices=["i8", "u8", "u16", "u32", "f32"])
    rs_read.add_argument("--timeout-ms", type=int, default=500)

    rs_write = sub.add_parser("robstride-write-param", help="write a RobStride parameter")
    _add_common_args(rs_write)
    rs_write.set_defaults(vendor="robstride")
    rs_write.set_defaults(model="rs-00", feedback_id="0xFF")
    rs_write.add_argument("--param-id", required=True)
    rs_write.add_argument("--type", required=True, choices=["i8", "u8", "u16", "u32", "f32"])
    rs_write.add_argument("--value", required=True)
    rs_write.add_argument("--verify", type=int, default=1)
    rs_write.add_argument("--timeout-ms", type=int, default=500)

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
        choices=["enable", "disable", "mit", "pos-vel", "vel", "force-pos", "ping"],
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
    args.model, args.feedback_id = _vendor_defaults(args.vendor, args.model, args.feedback_id)
    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    print(
        f"command=run vendor={args.vendor} transport={args.transport} channel={args.channel} "
        f"model={args.model} motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} mode={args.mode}"
    )

    with _open_controller(args, args.vendor) as ctrl:
        motor = _add_motor(ctrl, args.vendor, motor_id, feedback_id, args.model)
        try:
            if args.mode not in ("enable", "disable", "ping"):
                ctrl.enable_all()
                time.sleep(0.3)

            if args.ensure_mode and args.mode not in ("enable", "disable", "ping"):
                try:
                    if args.vendor == "robstride" and args.mode == "force-pos":
                        raise ValueError("robstride does not support force-pos")
                    motor.ensure_mode(_mode_to_enum(args.mode), args.ensure_timeout_ms)
                except Exception as e:
                    if args.ensure_strict:
                        raise
                    print(f"[warn] ensure_mode failed: {e}; continue anyway")

            for i in range(args.loop):
                if args.mode == "enable":
                    motor.enable()
                    if args.vendor == "damiao":
                        motor.request_feedback()
                elif args.mode == "disable":
                    motor.disable()
                    if args.vendor == "damiao":
                        motor.request_feedback()
                elif args.mode == "ping":
                    if args.vendor != "robstride":
                        raise ValueError("ping mode is only valid for RobStride")
                    device_id, responder_id = motor.robstride_ping()
                    print(f"#{i} ping device_id={device_id} responder_id={responder_id}")
                    break
                elif args.mode == "mit":
                    if args.vendor == "myactuator":
                        raise ValueError("myactuator does not support mit command")
                    motor.send_mit(args.pos, args.vel, args.kp, args.kd, args.tau)
                elif args.mode == "pos-vel":
                    if args.vendor == "robstride":
                        raise ValueError("robstride does not support pos-vel command")
                    motor.send_pos_vel(args.pos, args.vlim)
                elif args.mode == "vel":
                    if args.vendor == "hexfellow":
                        raise ValueError("hexfellow does not support vel command")
                    motor.send_vel(args.vel)
                elif args.mode == "force-pos":
                    if args.vendor in ("robstride", "myactuator", "hexfellow"):
                        raise ValueError(f"{args.vendor} does not support force-pos command")
                    motor.send_force_pos(args.pos, args.vlim, args.ratio)

                # Keep feedback state fresh during active control loops.
                if args.vendor == "damiao" and args.mode in ("mit", "pos-vel", "vel", "force-pos"):
                    motor.request_feedback()
                    try:
                        ctrl.poll_feedback_once()
                    except Exception:
                        # Best-effort polling; command loop should keep running.
                        pass

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
    if args.vendor != "damiao":
        raise ValueError("id-dump currently supports Damiao only")
    args.model, args.feedback_id = _vendor_defaults(args.vendor, args.model, args.feedback_id)
    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    rids = _parse_rids(args.rids)
    print(
        f"command=id-dump transport={args.transport} channel={args.channel} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X}"
    )
    ctrl = _open_controller(args, args.vendor)
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
    if args.vendor != "damiao":
        raise ValueError("id-set currently supports Damiao only")
    args.model, args.feedback_id = _vendor_defaults(args.vendor, args.model, args.feedback_id)
    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    new_motor_id = _parse_id(args.new_motor_id) if args.new_motor_id else motor_id
    new_feedback_id = _parse_id(args.new_feedback_id) if args.new_feedback_id else feedback_id
    print(
        f"command=id-set transport={args.transport} channel={args.channel} model={args.model} "
        f"old_motor_id=0x{motor_id:X} old_feedback_id=0x{feedback_id:X} "
        f"new_motor_id=0x{new_motor_id:X} new_feedback_id=0x{new_feedback_id:X}"
    )

    ctrl = _open_controller(args, args.vendor)
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

    verify_ctrl = _open_controller(args, args.vendor)
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


def _scan_damiao(args: argparse.Namespace, start_id: int, end_id: int) -> list[tuple[int, str]]:
    feedback_base = _parse_id(args.feedback_base)
    found: list[tuple[int, str]] = []
    print(
        f"[scan:damiao] channel={args.channel} model={args.model} "
        f"id_range=[0x{start_id:X},0x{end_id:X}] timeout_ms={args.timeout_ms}"
    )
    for mid in range(start_id, end_id + 1):
        fid = feedback_base + (mid & 0x0F)
        ctrl = _open_controller(args, "damiao")
        try:
            motor = ctrl.add_damiao_motor(mid, fid, args.model)
            try:
                esc_id = motor.get_register_u32(8, args.timeout_ms)
                mst_id = motor.get_register_u32(7, args.timeout_ms)
                found.append((mid, f"vendor=damiao esc_id=0x{esc_id:X} mst_id=0x{mst_id:X}"))
                print(f"[hit] vendor=damiao probe=0x{mid:02X} esc_id=0x{esc_id:X} mst_id=0x{mst_id:X}")
            except Exception:
                print(f"[.. ] vendor=damiao probe=0x{mid:02X} no reply")
            finally:
                motor.close()
        finally:
            ctrl.close_bus()
            ctrl.close()
    return found


def _scan_robstride(args: argparse.Namespace, start_id: int, end_id: int) -> list[tuple[int, str]]:
    feedback_ids = _parse_rids(args.feedback_ids)
    param_id = _parse_id(args.param_id)
    found: list[tuple[int, str]] = []
    print(
        f"[scan:robstride] channel={args.channel} model={args.model} "
        f"id_range=[0x{start_id:X},0x{end_id:X}] timeout_ms={args.timeout_ms} "
        f"feedback_ids={','.join(f'0x{x:X}' for x in feedback_ids)} param_id=0x{param_id:X}"
    )
    for mid in range(start_id, end_id + 1):
        hit_meta = None
        for fid in feedback_ids:
            ctrl = Controller(args.channel)
            try:
                motor = ctrl.add_robstride_motor(mid, fid, args.model)
                try:
                    try:
                        device_id, responder_id = motor.robstride_ping()
                        hit_meta = (
                            f"vendor=robstride via=ping feedback_id=0x{fid:X} "
                            f"device_id={device_id} responder_id={responder_id}"
                        )
                        break
                    except Exception:
                        value = motor.robstride_get_param_f32(param_id, args.param_timeout_ms)
                        hit_meta = (
                            f"vendor=robstride via=read-param feedback_id=0x{fid:X} "
                            f"param_id=0x{param_id:X} value={value}"
                        )
                        break
                finally:
                    motor.close()
            finally:
                ctrl.close_bus()
                ctrl.close()
        if hit_meta is None:
            print(f"[.. ] vendor=robstride probe=0x{mid:02X} no reply")
        else:
            found.append((mid, hit_meta))
            print(f"[hit] probe=0x{mid:02X} {hit_meta}")
    return found


def _scan_myactuator(args: argparse.Namespace, start_id: int, end_id: int) -> list[tuple[int, str]]:
    found: list[tuple[int, str]] = []
    lo = max(1, start_id)
    hi = min(32, end_id)
    print(
        f"[scan:myactuator] channel={args.channel} model={args.model} "
        f"id_range=[0x{lo:X},0x{hi:X}] timeout_ms={args.timeout_ms}"
    )
    for mid in range(lo, hi + 1):
        fid = 0x240 + mid
        ctrl = Controller(args.channel)
        try:
            motor = ctrl.add_myactuator_motor(mid, fid, args.model)
            try:
                try:
                    motor.request_feedback()
                    time.sleep(min(max(args.timeout_ms, 10), 300) / 1000.0)
                    ctrl.poll_feedback_once()
                    st = motor.get_state()
                    if st is None:
                        raise RuntimeError("no feedback")
                    meta = (
                        f"vendor=myactuator feedback_id=0x{fid:X} "
                        f"temp={st.t_mos:.1f}C vel={st.vel:+.3f}rad/s angle={st.pos:+.3f}rad"
                    )
                    found.append((mid, meta))
                    print(f"[hit] probe=0x{mid:02X} {meta}")
                except Exception:
                    print(f"[.. ] vendor=myactuator probe=0x{mid:02X} no reply")
            finally:
                motor.close()
        finally:
            ctrl.close_bus()
            ctrl.close()
    return found


def _scan_hightorque(args: argparse.Namespace, start_id: int, end_id: int) -> list[tuple[int, str]]:
    found: list[tuple[int, str]] = []
    lo = max(1, start_id)
    hi = min(127, end_id)
    print(
        f"[scan:hightorque] channel={args.channel} model={args.model} "
        f"id_range=[0x{lo:X},0x{hi:X}] timeout_ms={args.timeout_ms}"
    )
    for mid in range(lo, hi + 1):
        ctrl = Controller(args.channel)
        try:
            motor = ctrl.add_hightorque_motor(mid, 0x01, args.model)
            try:
                try:
                    motor.request_feedback()
                    st = motor.get_state()
                    if st is None:
                        raise RuntimeError("no feedback")
                    meta = (
                        f"vendor=hightorque pos={st.pos:+.4f}rad "
                        f"vel={st.vel:+.4f}rad/s torq={st.torq:+.3f}Nm"
                    )
                    found.append((mid, meta))
                    print(f"[hit] probe=0x{mid:02X} {meta}")
                except Exception:
                    print(f"[.. ] vendor=hightorque probe=0x{mid:02X} no reply")
            finally:
                motor.close()
        finally:
            ctrl.close_bus()
            ctrl.close()
    return found


def _scan_command(args: argparse.Namespace) -> None:
    start_id = _parse_id(args.start_id)
    end_id = _parse_id(args.end_id)
    if end_id < start_id:
        raise ValueError("end-id must be >= start-id")
    if args.transport == "dm-serial" and args.vendor != "damiao":
        raise ValueError("scan with transport=dm-serial currently supports --vendor damiao only")
    resolved_model, _ = _vendor_defaults(args.vendor if args.vendor != "all" else "damiao", args.model, "0x11")
    args.model = resolved_model
    print(
        f"command=scan vendor={args.vendor} transport={args.transport} channel={args.channel} model={args.model} "
        f"id_range=[0x{start_id:X},0x{end_id:X}] timeout_ms={args.timeout_ms}"
    )

    found: list[tuple[int, str]] = []
    damiao_model = args.model
    myactuator_model = "X8" if args.model == "4340" else args.model
    robstride_model = "rs-00" if args.model == "4340" else args.model
    hightorque_model = "hightorque" if args.model == "4340" else args.model
    if args.vendor in ("damiao", "all"):
        args.model = damiao_model
        found.extend(_scan_damiao(args, start_id, end_id))
    if args.vendor in ("myactuator", "all"):
        args.model = myactuator_model
        found.extend(_scan_myactuator(args, start_id, end_id))
    if args.vendor in ("robstride", "all"):
        args.model = robstride_model
        found.extend(_scan_robstride(args, start_id, end_id))
    if args.vendor in ("hightorque", "all"):
        args.model = hightorque_model
        found.extend(_scan_hightorque(args, start_id, end_id))

    print(f"scan done: {len(found)} motor(s) found")
    for probe, meta in found:
        print(f"  probe=0x{probe:02X} {meta}")


def _robstride_read_param_command(args: argparse.Namespace) -> None:
    if args.vendor != "robstride":
        raise ValueError("robstride-read-param is only valid for --vendor robstride")
    args.model, args.feedback_id = _vendor_defaults(args.vendor, args.model, args.feedback_id)
    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    param_id = _parse_id(args.param_id)
    with Controller(args.channel) as ctrl:
        motor = ctrl.add_robstride_motor(motor_id, feedback_id, args.model)
        try:
            if args.type == "i8":
                value = motor.robstride_get_param_i8(param_id, args.timeout_ms)
            elif args.type == "u8":
                value = motor.robstride_get_param_u8(param_id, args.timeout_ms)
            elif args.type == "u16":
                value = motor.robstride_get_param_u16(param_id, args.timeout_ms)
            elif args.type == "u32":
                value = motor.robstride_get_param_u32(param_id, args.timeout_ms)
            else:
                value = motor.robstride_get_param_f32(param_id, args.timeout_ms)
            print(
                f"command=robstride-read-param channel={args.channel} model={args.model} "
                f"motor_id=0x{motor_id:X} param_id=0x{param_id:X} type={args.type} value={value}"
            )
        finally:
            motor.close()


def _robstride_write_param_command(args: argparse.Namespace) -> None:
    if args.vendor != "robstride":
        raise ValueError("robstride-write-param is only valid for --vendor robstride")
    args.model, args.feedback_id = _vendor_defaults(args.vendor, args.model, args.feedback_id)
    motor_id = _parse_id(args.motor_id)
    feedback_id = _parse_id(args.feedback_id)
    param_id = _parse_id(args.param_id)
    with Controller(args.channel) as ctrl:
        motor = ctrl.add_robstride_motor(motor_id, feedback_id, args.model)
        try:
            if args.type == "i8":
                motor.robstride_write_param_i8(param_id, int(args.value, 0))
                verify = motor.robstride_get_param_i8(param_id, args.timeout_ms) if args.verify else None
            elif args.type == "u8":
                motor.robstride_write_param_u8(param_id, int(args.value, 0))
                verify = motor.robstride_get_param_u8(param_id, args.timeout_ms) if args.verify else None
            elif args.type == "u16":
                motor.robstride_write_param_u16(param_id, int(args.value, 0))
                verify = motor.robstride_get_param_u16(param_id, args.timeout_ms) if args.verify else None
            elif args.type == "u32":
                motor.robstride_write_param_u32(param_id, int(args.value, 0))
                verify = motor.robstride_get_param_u32(param_id, args.timeout_ms) if args.verify else None
            else:
                motor.robstride_write_param_f32(param_id, float(args.value))
                verify = motor.robstride_get_param_f32(param_id, args.timeout_ms) if args.verify else None
            print(
                f"command=robstride-write-param channel={args.channel} model={args.model} "
                f"motor_id=0x{motor_id:X} param_id=0x{param_id:X} type={args.type} "
                f"value={args.value} verify={verify}"
            )
        finally:
            motor.close()


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
    elif args.command == "robstride-read-param":
        _robstride_read_param_command(args)
    elif args.command == "robstride-write-param":
        _robstride_write_param_command(args)
    else:
        raise RuntimeError(f"unknown command: {args.command}")


if __name__ == "__main__":
    main()
