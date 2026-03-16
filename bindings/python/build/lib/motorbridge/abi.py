import ctypes
import ctypes.util
import os
from ctypes import POINTER, Structure, c_char_p, c_float, c_int32, c_uint8, c_uint16, c_uint32, c_void_p
from pathlib import Path

from .errors import AbiLoadError


class CState(Structure):
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


def _candidate_lib_paths() -> list[Path]:
    candidates: list[Path] = []
    env = os.getenv("MOTORBRIDGE_LIB")
    if env:
        candidates.append(Path(env).expanduser())

    here = Path(__file__).resolve()
    pkg_lib = here.parent / "lib"
    if pkg_lib.exists():
        candidates.extend(
            [
                pkg_lib / "libmotor_abi.so",
                pkg_lib / "libmotor_abi.dylib",
                pkg_lib / "motor_abi.dll",
            ]
        )

    repo_root = here.parents[4]
    candidates.extend(
        [
            repo_root / "target" / "release" / "libmotor_abi.so",
            repo_root / "target" / "release" / "libmotor_abi.dylib",
            repo_root / "target" / "release" / "motor_abi.dll",
        ]
    )

    cwd = Path.cwd()
    candidates.extend(
        [
            cwd / "target" / "release" / "libmotor_abi.so",
            cwd / "target" / "release" / "libmotor_abi.dylib",
            cwd / "target" / "release" / "motor_abi.dll",
        ]
    )
    return candidates


def _load_library() -> ctypes.CDLL:
    tried: list[str] = []
    for p in _candidate_lib_paths():
        tried.append(str(p))
        if p.exists():
            return ctypes.CDLL(str(p))

    found = ctypes.util.find_library("motor_abi")
    if found:
        return ctypes.CDLL(found)

    raise AbiLoadError(
        "Failed to load motor_abi shared library. Tried:\n"
        + "\n".join(f"- {x}" for x in tried)
        + "\nHint: build ABI first: cargo build -p motor_abi --release"
    )


class Abi:
    def __init__(self) -> None:
        self.lib = _load_library()
        self._bind()

    def _bind(self) -> None:
        lib = self.lib

        lib.motor_last_error_message.restype = c_char_p

        lib.motor_controller_new_socketcan.argtypes = [c_char_p]
        lib.motor_controller_new_socketcan.restype = c_void_p
        lib.motor_controller_free.argtypes = [c_void_p]
        lib.motor_controller_poll_feedback_once.argtypes = [c_void_p]
        lib.motor_controller_poll_feedback_once.restype = c_int32
        lib.motor_controller_enable_all.argtypes = [c_void_p]
        lib.motor_controller_enable_all.restype = c_int32
        lib.motor_controller_disable_all.argtypes = [c_void_p]
        lib.motor_controller_disable_all.restype = c_int32
        lib.motor_controller_shutdown.argtypes = [c_void_p]
        lib.motor_controller_shutdown.restype = c_int32

        lib.motor_controller_add_damiao_motor.argtypes = [c_void_p, c_uint16, c_uint16, c_char_p]
        lib.motor_controller_add_damiao_motor.restype = c_void_p

        lib.motor_handle_free.argtypes = [c_void_p]
        lib.motor_handle_enable.argtypes = [c_void_p]
        lib.motor_handle_enable.restype = c_int32
        lib.motor_handle_disable.argtypes = [c_void_p]
        lib.motor_handle_disable.restype = c_int32
        lib.motor_handle_clear_error.argtypes = [c_void_p]
        lib.motor_handle_clear_error.restype = c_int32
        lib.motor_handle_set_zero_position.argtypes = [c_void_p]
        lib.motor_handle_set_zero_position.restype = c_int32
        lib.motor_handle_ensure_mode.argtypes = [c_void_p, c_uint32, c_uint32]
        lib.motor_handle_ensure_mode.restype = c_int32

        lib.motor_handle_send_mit.argtypes = [c_void_p, c_float, c_float, c_float, c_float, c_float]
        lib.motor_handle_send_mit.restype = c_int32
        lib.motor_handle_send_pos_vel.argtypes = [c_void_p, c_float, c_float]
        lib.motor_handle_send_pos_vel.restype = c_int32
        lib.motor_handle_send_vel.argtypes = [c_void_p, c_float]
        lib.motor_handle_send_vel.restype = c_int32
        lib.motor_handle_send_force_pos.argtypes = [c_void_p, c_float, c_float, c_float]
        lib.motor_handle_send_force_pos.restype = c_int32

        lib.motor_handle_store_parameters.argtypes = [c_void_p]
        lib.motor_handle_store_parameters.restype = c_int32
        lib.motor_handle_request_feedback.argtypes = [c_void_p]
        lib.motor_handle_request_feedback.restype = c_int32
        lib.motor_handle_set_can_timeout_ms.argtypes = [c_void_p, c_uint32]
        lib.motor_handle_set_can_timeout_ms.restype = c_int32

        lib.motor_handle_write_register_f32.argtypes = [c_void_p, c_uint8, c_float]
        lib.motor_handle_write_register_f32.restype = c_int32
        lib.motor_handle_write_register_u32.argtypes = [c_void_p, c_uint8, c_uint32]
        lib.motor_handle_write_register_u32.restype = c_int32
        lib.motor_handle_get_register_f32.argtypes = [c_void_p, c_uint8, c_uint32, POINTER(c_float)]
        lib.motor_handle_get_register_f32.restype = c_int32
        lib.motor_handle_get_register_u32.argtypes = [c_void_p, c_uint8, c_uint32, POINTER(c_uint32)]
        lib.motor_handle_get_register_u32.restype = c_int32

        lib.motor_handle_get_state.argtypes = [c_void_p, POINTER(CState)]
        lib.motor_handle_get_state.restype = c_int32


_abi_singleton: Abi | None = None


def get_abi() -> Abi:
    global _abi_singleton
    if _abi_singleton is None:
        _abi_singleton = Abi()
    return _abi_singleton
