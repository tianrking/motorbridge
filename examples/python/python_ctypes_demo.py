import argparse
import ctypes
import time
from ctypes import (
    POINTER,
    Structure,
    c_char_p,
    c_float,
    c_int32,
    c_int8,
    c_uint8,
    c_uint16,
    c_uint32,
    c_void_p,
)


class MotorState(Structure):
    _fields_ = [
        ("has_value", c_int32),
        ("can_id", c_uint8),
        ("arbitration_id", c_uint32),
        ("status_code", c_uint8),
        ("pos", c_float),
        ("vel", c_float),
        ("torq", c_float),
        ("t_mos", c_float),
        ("t_rotor", c_float),
    ]


lib = ctypes.CDLL("target/release/libmotor_abi.so")

lib.motor_last_error_message.restype = c_char_p

lib.motor_controller_new_socketcan.argtypes = [c_char_p]
lib.motor_controller_new_socketcan.restype = c_void_p
lib.motor_controller_new_dm_serial.argtypes = [c_char_p, c_uint32]
lib.motor_controller_new_dm_serial.restype = c_void_p
lib.motor_controller_enable_all.argtypes = [c_void_p]
lib.motor_controller_enable_all.restype = c_int32
lib.motor_controller_shutdown.argtypes = [c_void_p]
lib.motor_controller_shutdown.restype = c_int32
lib.motor_controller_free.argtypes = [c_void_p]
lib.motor_controller_add_damiao_motor.argtypes = [c_void_p, c_uint16, c_uint16, c_char_p]
lib.motor_controller_add_damiao_motor.restype = c_void_p
lib.motor_controller_add_robstride_motor.argtypes = [c_void_p, c_uint16, c_uint16, c_char_p]
lib.motor_controller_add_robstride_motor.restype = c_void_p

lib.motor_handle_ensure_mode.argtypes = [c_void_p, c_uint32, c_uint32]
lib.motor_handle_ensure_mode.restype = c_int32
lib.motor_handle_enable.argtypes = [c_void_p]
lib.motor_handle_enable.restype = c_int32
lib.motor_handle_disable.argtypes = [c_void_p]
lib.motor_handle_disable.restype = c_int32
lib.motor_handle_send_mit.argtypes = [c_void_p, c_float, c_float, c_float, c_float, c_float]
lib.motor_handle_send_mit.restype = c_int32
lib.motor_handle_send_pos_vel.argtypes = [c_void_p, c_float, c_float]
lib.motor_handle_send_pos_vel.restype = c_int32
lib.motor_handle_send_vel.argtypes = [c_void_p, c_float]
lib.motor_handle_send_vel.restype = c_int32
lib.motor_handle_send_force_pos.argtypes = [c_void_p, c_float, c_float, c_float]
lib.motor_handle_send_force_pos.restype = c_int32
lib.motor_handle_request_feedback.argtypes = [c_void_p]
lib.motor_handle_request_feedback.restype = c_int32
lib.motor_handle_get_state.argtypes = [c_void_p, POINTER(MotorState)]
lib.motor_handle_get_state.restype = c_int32
lib.motor_handle_robstride_ping.argtypes = [c_void_p, POINTER(c_uint8), POINTER(c_uint8)]
lib.motor_handle_robstride_ping.restype = c_int32
lib.motor_handle_robstride_write_param_i8.argtypes = [c_void_p, c_uint16, c_int8]
lib.motor_handle_robstride_write_param_i8.restype = c_int32
lib.motor_handle_robstride_write_param_u8.argtypes = [c_void_p, c_uint16, c_uint8]
lib.motor_handle_robstride_write_param_u8.restype = c_int32
lib.motor_handle_robstride_write_param_u16.argtypes = [c_void_p, c_uint16, c_uint16]
lib.motor_handle_robstride_write_param_u16.restype = c_int32
lib.motor_handle_robstride_write_param_u32.argtypes = [c_void_p, c_uint16, c_uint32]
lib.motor_handle_robstride_write_param_u32.restype = c_int32
lib.motor_handle_robstride_write_param_f32.argtypes = [c_void_p, c_uint16, c_float]
lib.motor_handle_robstride_write_param_f32.restype = c_int32
lib.motor_handle_robstride_get_param_i8.argtypes = [c_void_p, c_uint16, c_uint32, POINTER(c_int8)]
lib.motor_handle_robstride_get_param_i8.restype = c_int32
lib.motor_handle_robstride_get_param_u8.argtypes = [c_void_p, c_uint16, c_uint32, POINTER(c_uint8)]
lib.motor_handle_robstride_get_param_u8.restype = c_int32
lib.motor_handle_robstride_get_param_u16.argtypes = [c_void_p, c_uint16, c_uint32, POINTER(c_uint16)]
lib.motor_handle_robstride_get_param_u16.restype = c_int32
lib.motor_handle_robstride_get_param_u32.argtypes = [c_void_p, c_uint16, c_uint32, POINTER(c_uint32)]
lib.motor_handle_robstride_get_param_u32.restype = c_int32
lib.motor_handle_robstride_get_param_f32.argtypes = [c_void_p, c_uint16, c_uint32, POINTER(c_float)]
lib.motor_handle_robstride_get_param_f32.restype = c_int32
lib.motor_handle_free.argtypes = [c_void_p]


def must_ok(rc: int, what: str) -> None:
    if rc != 0:
        raise RuntimeError(f"{what} failed: {lib.motor_last_error_message().decode()}")


def parse_id(text: str) -> int:
    return int(text, 0)


def apply_vendor_defaults(args) -> None:
    if args.vendor == "robstride":
        if args.model == "4340":
            args.model = "rs-00"
        if args.feedback_id == "0x11":
            args.feedback_id = "0xFF"


def read_robstride_param(motor: int, param_id: int, param_type: str, timeout_ms: int):
    if param_type == "i8":
        out = c_int8(0)
        must_ok(
            lib.motor_handle_robstride_get_param_i8(motor, param_id, timeout_ms, ctypes.byref(out)),
            "robstride_get_param_i8",
        )
        return int(out.value)
    if param_type == "u8":
        out = c_uint8(0)
        must_ok(
            lib.motor_handle_robstride_get_param_u8(motor, param_id, timeout_ms, ctypes.byref(out)),
            "robstride_get_param_u8",
        )
        return int(out.value)
    if param_type == "u16":
        out = c_uint16(0)
        must_ok(
            lib.motor_handle_robstride_get_param_u16(motor, param_id, timeout_ms, ctypes.byref(out)),
            "robstride_get_param_u16",
        )
        return int(out.value)
    if param_type == "u32":
        out = c_uint32(0)
        must_ok(
            lib.motor_handle_robstride_get_param_u32(motor, param_id, timeout_ms, ctypes.byref(out)),
            "robstride_get_param_u32",
        )
        return int(out.value)

    out = c_float(0.0)
    must_ok(
        lib.motor_handle_robstride_get_param_f32(motor, param_id, timeout_ms, ctypes.byref(out)),
        "robstride_get_param_f32",
    )
    return float(out.value)


def write_robstride_param(motor: int, param_id: int, param_type: str, value: str) -> None:
    if param_type == "i8":
        must_ok(
            lib.motor_handle_robstride_write_param_i8(motor, param_id, int(value, 0)),
            "robstride_write_param_i8",
        )
        return
    if param_type == "u8":
        must_ok(
            lib.motor_handle_robstride_write_param_u8(motor, param_id, int(value, 0)),
            "robstride_write_param_u8",
        )
        return
    if param_type == "u16":
        must_ok(
            lib.motor_handle_robstride_write_param_u16(motor, param_id, int(value, 0)),
            "robstride_write_param_u16",
        )
        return
    if param_type == "u32":
        must_ok(
            lib.motor_handle_robstride_write_param_u32(motor, param_id, int(value, 0)),
            "robstride_write_param_u32",
        )
        return

    must_ok(
        lib.motor_handle_robstride_write_param_f32(motor, param_id, float(value)),
        "robstride_write_param_f32",
    )


def print_state(prefix: str, motor: int) -> None:
    state = MotorState()
    must_ok(lib.motor_handle_get_state(motor, ctypes.byref(state)), "get_state")
    if state.has_value:
        print(
            f"{prefix} pos={state.pos:+.3f} vel={state.vel:+.3f} "
            f"torq={state.torq:+.3f} status={state.status_code} "
            f"arb=0x{state.arbitration_id:X}"
        )
    else:
        print(f"{prefix} no feedback yet")


def main() -> None:
    parser = argparse.ArgumentParser(description="motor_abi ctypes demo (Damiao + RobStride)")
    parser.add_argument("--transport", choices=["socketcan", "dm-serial"], default="socketcan")
    parser.add_argument("--channel", default="can0")
    parser.add_argument("--serial-port", default="/dev/ttyACM0")
    parser.add_argument("--serial-baud", type=int, default=921600)
    parser.add_argument("--vendor", choices=["damiao", "robstride"], default="damiao")
    parser.add_argument("--model", default="4340")
    parser.add_argument("--motor-id", default="0x01")
    parser.add_argument("--feedback-id", default="0x11")
    parser.add_argument(
        "--mode",
        default="mit",
        choices=[
            "enable",
            "disable",
            "mit",
            "pos-vel",
            "vel",
            "force-pos",
            "ping",
            "read-param",
            "write-param",
        ],
        help="control mode",
    )
    parser.add_argument("--loop", type=int, default=100)
    parser.add_argument("--dt-ms", type=int, default=20)
    parser.add_argument("--ensure-mode", type=int, default=1, help="1/0")
    parser.add_argument("--ensure-timeout-ms", type=int, default=1000)
    parser.add_argument(
        "--ensure-strict",
        type=int,
        default=0,
        help="1/0, fail fast if ensure_mode fails (default: warn and continue)",
    )
    parser.add_argument("--print-state", type=int, default=1, help="1/0")
    parser.add_argument("--pos", type=float, default=0.0)
    parser.add_argument("--vel", type=float, default=0.0)
    parser.add_argument("--kp", type=float, default=30.0)
    parser.add_argument("--kd", type=float, default=1.0)
    parser.add_argument("--tau", type=float, default=0.0)
    parser.add_argument("--vlim", type=float, default=1.0)
    parser.add_argument("--ratio", type=float, default=0.3)
    parser.add_argument("--param-id", default="0x7019")
    parser.add_argument("--param-type", choices=["i8", "u8", "u16", "u32", "f32"], default="f32")
    parser.add_argument("--param-value", default="0")
    parser.add_argument("--param-timeout-ms", type=int, default=1000)
    args = parser.parse_args()
    apply_vendor_defaults(args)

    motor_id = parse_id(args.motor_id)
    feedback_id = parse_id(args.feedback_id)
    param_id = parse_id(args.param_id)

    print(
        f"vendor={args.vendor} transport={args.transport} channel={args.channel} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} mode={args.mode}"
    )

    if args.transport == "dm-serial":
        if args.vendor != "damiao":
            raise RuntimeError("transport=dm-serial is available for damiao only in this demo")
        ctrl = lib.motor_controller_new_dm_serial(
            args.serial_port.encode(), int(args.serial_baud)
        )
    else:
        ctrl = lib.motor_controller_new_socketcan(args.channel.encode())
    if not ctrl:
        raise RuntimeError(lib.motor_last_error_message().decode())

    if args.vendor == "damiao":
        motor = lib.motor_controller_add_damiao_motor(
            ctrl, motor_id, feedback_id, args.model.encode()
        )
    else:
        motor = lib.motor_controller_add_robstride_motor(
            ctrl, motor_id, feedback_id, args.model.encode()
        )
    if not motor:
        raise RuntimeError(lib.motor_last_error_message().decode())

    try:
        if args.vendor == "damiao" and args.mode in {"ping", "read-param", "write-param"}:
            raise RuntimeError("Damiao path does not support robstride-only modes")
        if args.vendor == "robstride" and args.mode in {"pos-vel", "force-pos"}:
            raise RuntimeError("RobStride demo supports ping/enable/disable/mit/vel/read-param/write-param")

        if args.mode in {"ping", "read-param", "write-param"}:
            pass
        elif args.mode not in ("enable", "disable"):
            must_ok(lib.motor_controller_enable_all(ctrl), "enable_all")
            time.sleep(0.3)

        if args.ensure_mode and args.mode not in ("enable", "disable", "ping", "read-param", "write-param"):
            mode_map = {"mit": 1, "pos-vel": 2, "vel": 3, "force-pos": 4}
            rc = lib.motor_handle_ensure_mode(
                motor, mode_map[args.mode], args.ensure_timeout_ms
            )
            if rc != 0:
                msg = lib.motor_last_error_message().decode()
                if args.ensure_strict:
                    raise RuntimeError(f"ensure_mode failed: {msg}")
                print(f"[warn] ensure_mode failed: {msg}; continue anyway")

        if args.mode == "ping":
            device_id = c_uint8(0)
            responder_id = c_uint8(0)
            must_ok(
                lib.motor_handle_robstride_ping(
                    motor, ctypes.byref(device_id), ctypes.byref(responder_id)
                ),
                "robstride_ping",
            )
            print(
                f"ping ok device_id={int(device_id.value)} responder_id={int(responder_id.value)}"
            )
            print_state("[state]", motor)
            return

        if args.mode == "read-param":
            value = read_robstride_param(motor, param_id, args.param_type, args.param_timeout_ms)
            print(f"param 0x{param_id:04X} ({args.param_type}) = {value}")
            print_state("[state]", motor)
            return

        if args.mode == "write-param":
            write_robstride_param(motor, param_id, args.param_type, args.param_value)
            print(f"wrote param 0x{param_id:04X} ({args.param_type}) <- {args.param_value}")
            value = read_robstride_param(motor, param_id, args.param_type, args.param_timeout_ms)
            print(f"readback param 0x{param_id:04X} ({args.param_type}) = {value}")
            print_state("[state]", motor)
            return

        for i in range(args.loop):
            if args.mode == "enable":
                must_ok(lib.motor_handle_enable(motor), "enable")
                lib.motor_handle_request_feedback(motor)
            elif args.mode == "disable":
                must_ok(lib.motor_handle_disable(motor), "disable")
                lib.motor_handle_request_feedback(motor)
            elif args.mode == "mit":
                must_ok(
                    lib.motor_handle_send_mit(
                        motor, args.pos, args.vel, args.kp, args.kd, args.tau
                    ),
                    "send_mit",
                )
            elif args.mode == "pos-vel":
                must_ok(lib.motor_handle_send_pos_vel(motor, args.pos, args.vlim), "send_pos_vel")
            elif args.mode == "vel":
                must_ok(lib.motor_handle_send_vel(motor, args.vel), "send_vel")
            elif args.mode == "force-pos":
                must_ok(
                    lib.motor_handle_send_force_pos(motor, args.pos, args.vlim, args.ratio),
                    "send_force_pos",
                )

            if args.print_state:
                print_state(f"#{i}", motor)
            time.sleep(max(args.dt_ms, 0) / 1000.0)
    finally:
        lib.motor_controller_shutdown(ctrl)
        lib.motor_handle_free(motor)
        lib.motor_controller_free(ctrl)


if __name__ == "__main__":
    main()
