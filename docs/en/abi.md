# ABI Guide (`motor_abi`)

## Build

```bash
cargo build -p motor_abi --release
```

Outputs:

- Linux: `target/release/libmotor_abi.so`, `libmotor_abi.a`
- macOS: `target/release/libmotor_abi.dylib`, `libmotor_abi.a`
- Windows: `target/release/motor_abi.dll`, `motor_abi.lib`

Header:

- `motor_abi/include/motor_abi.h`

## Return Code Convention

- `0` = success
- `-1` = failure
- error string from `motor_last_error_message()`

## Controller APIs

- `motor_controller_new_socketcan`
- `motor_controller_poll_feedback_once`
- `motor_controller_enable_all`
- `motor_controller_disable_all`
- `motor_controller_shutdown`
- `motor_controller_close_bus`
- `motor_controller_free`

Lifecycle recommendation:

- Use `shutdown` for explicit stop/disable workflows.
- Use `close_bus` for query/scan/id-tooling sessions where implicit shutdown is undesirable.
- Then call `free` to release resources.

## Motor Handle APIs

- create/free: `motor_controller_add_damiao_motor`, `motor_handle_free`
- control: `motor_handle_enable`, `motor_handle_disable`, `motor_handle_clear_error`, `motor_handle_set_zero_position`
- mode: `motor_handle_ensure_mode`
- command: `motor_handle_send_mit`, `motor_handle_send_pos_vel`, `motor_handle_send_vel`, `motor_handle_send_force_pos`
- register: `motor_handle_write_register_f32/u32`, `motor_handle_get_register_f32/u32`
- ops/state: `motor_handle_store_parameters`, `motor_handle_request_feedback`, `motor_handle_set_can_timeout_ms`, `motor_handle_get_state`

## Mode Values

For `motor_handle_ensure_mode(motor, mode, timeout_ms)`:

- `1 = MIT`
- `2 = POS_VEL`
- `3 = VEL`
- `4 = FORCE_POS`

## C/C++/Python References

- C example: `examples/c/c_abi_demo.c`
- C++ example: `examples/cpp/cpp_abi_demo.cpp`
- Python ctypes example: `examples/python/python_ctypes_demo.py`
- Python SDK wrapper: `bindings/python`
