import ctypes
import argparse
import time
from ctypes import c_char_p, c_float, c_int32, c_uint8, c_uint16, c_uint32, c_void_p, POINTER, Structure


class MotorState(Structure):
    _fields_ = [
        ("has_value", c_int32),
        ("can_id", c_uint8),
        ("arbitration_id", c_uint16),
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
lib.motor_controller_enable_all.argtypes = [c_void_p]
lib.motor_controller_enable_all.restype = c_int32
lib.motor_controller_shutdown.argtypes = [c_void_p]
lib.motor_controller_shutdown.restype = c_int32
lib.motor_controller_free.argtypes = [c_void_p]
lib.motor_controller_add_damiao_motor.argtypes = [c_void_p, c_uint16, c_uint16, c_char_p]
lib.motor_controller_add_damiao_motor.restype = c_void_p

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
lib.motor_handle_free.argtypes = [c_void_p]


def must_ok(rc: int, what: str) -> None:
    if rc != 0:
        raise RuntimeError(f"{what} failed: {lib.motor_last_error_message().decode()}")


def main() -> None:
    parser = argparse.ArgumentParser(description="motor_abi ctypes demo (multi-mode)")
    parser.add_argument("--channel", default="can0")
    parser.add_argument("--model", default="4340")
    parser.add_argument("--motor-id", default="0x01")
    parser.add_argument("--feedback-id", default="0x11")
    parser.add_argument(
        "--mode",
        default="mit",
        choices=["enable", "disable", "mit", "pos-vel", "vel", "force-pos"],
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
    args = parser.parse_args()

    motor_id = int(args.motor_id, 0)
    feedback_id = int(args.feedback_id, 0)

    print(
        f"channel={args.channel} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X} mode={args.mode}"
    )

    ctrl = lib.motor_controller_new_socketcan(args.channel.encode())
    if not ctrl:
        raise RuntimeError(lib.motor_last_error_message().decode())

    motor = lib.motor_controller_add_damiao_motor(
        ctrl, motor_id, feedback_id, args.model.encode()
    )
    if not motor:
        raise RuntimeError(lib.motor_last_error_message().decode())

    try:
        if args.mode not in ("enable", "disable"):
            must_ok(lib.motor_controller_enable_all(ctrl), "enable_all")
            time.sleep(0.3)

        if args.ensure_mode and args.mode not in ("enable", "disable"):
            mode_map = {"mit": 1, "pos-vel": 2, "vel": 3, "force-pos": 4}
            rc = lib.motor_handle_ensure_mode(
                motor, mode_map[args.mode], args.ensure_timeout_ms
            )
            if rc != 0:
                msg = lib.motor_last_error_message().decode()
                if args.ensure_strict:
                    raise RuntimeError(f"ensure_mode failed: {msg}")
                print(f"[warn] ensure_mode failed: {msg}; continue anyway")

        state = MotorState()
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
                must_ok(lib.motor_handle_get_state(motor, ctypes.byref(state)), "get_state")
                if state.has_value:
                    print(
                        f"#{i} pos={state.pos:+.3f} vel={state.vel:+.3f} "
                        f"torq={state.torq:+.3f} status={state.status_code}"
                    )
                else:
                    print(f"#{i} no feedback yet")
            time.sleep(max(args.dt_ms, 0) / 1000.0)
    finally:
        lib.motor_controller_shutdown(ctrl)
        lib.motor_handle_free(motor)
        lib.motor_controller_free(ctrl)


if __name__ == "__main__":
    main()
