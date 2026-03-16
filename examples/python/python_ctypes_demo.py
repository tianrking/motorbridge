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
lib.motor_handle_send_mit.argtypes = [c_void_p, c_float, c_float, c_float, c_float, c_float]
lib.motor_handle_send_mit.restype = c_int32
lib.motor_handle_get_state.argtypes = [c_void_p, POINTER(MotorState)]
lib.motor_handle_get_state.restype = c_int32
lib.motor_handle_free.argtypes = [c_void_p]


def must_ok(rc: int, what: str) -> None:
    if rc != 0:
        raise RuntimeError(f"{what} failed: {lib.motor_last_error_message().decode()}")


def main() -> None:
    parser = argparse.ArgumentParser(description="motor_abi ctypes demo")
    parser.add_argument("--channel", default="can0")
    parser.add_argument("--model", default="4340")
    parser.add_argument("--motor-id", default="0x01")
    parser.add_argument("--feedback-id", default="0x11")
    args = parser.parse_args()

    motor_id = int(args.motor_id, 0)
    feedback_id = int(args.feedback_id, 0)

    print(
        f"channel={args.channel} model={args.model} "
        f"motor_id=0x{motor_id:X} feedback_id=0x{feedback_id:X}"
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
        must_ok(lib.motor_controller_enable_all(ctrl), "enable_all")
        time.sleep(0.5)

        # 1 = MIT
        must_ok(lib.motor_handle_ensure_mode(motor, 1, 1000), "ensure_mode")

        state = MotorState()
        for _ in range(200):
            must_ok(lib.motor_handle_send_mit(motor, 0.0, 0.0, 30.0, 1.0, 0.0), "send_mit")
            must_ok(lib.motor_handle_get_state(motor, ctypes.byref(state)), "get_state")
            if state.has_value:
                print(f"pos={state.pos:+.3f} vel={state.vel:+.3f} torq={state.torq:+.3f}")
            time.sleep(0.02)
    finally:
        lib.motor_controller_shutdown(ctrl)
        lib.motor_handle_free(motor)
        lib.motor_controller_free(ctrl)


if __name__ == "__main__":
    main()
