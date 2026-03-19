# ABI Guide (`motor_abi`)

## Build

```bash
cargo build -p motor_abi --release
```

Artifacts:

- Linux: `target/release/libmotor_abi.so`, `libmotor_abi.a`
- Windows: `target/release/motor_abi.dll`, `motor_abi.lib`
- Header: `motor_abi/include/motor_abi.h`

Distribution formats:

- Linux x86_64: `.deb` and `.tar.gz`
- Windows x86_64: `.zip` (no `.deb` on Windows)

## Vendor Entry Points

- Damiao: `motor_controller_add_damiao_motor(...)`
- RobStride: `motor_controller_add_robstride_motor(...)`

Both vendor handles share the common control surface:

- `motor_handle_enable`
- `motor_handle_disable`
- `motor_handle_send_mit`
- `motor_handle_send_vel`
- `motor_handle_get_state`

Damiao-only flows keep using:

- `motor_handle_send_pos_vel`
- `motor_handle_send_force_pos`
- Damiao register read/write helpers

RobStride-specific ABI additions:

- `motor_handle_robstride_ping`
- `motor_handle_robstride_write_param_i8/u8/u16/u32/f32`
- `motor_handle_robstride_get_param_i8/u8/u16/u32/f32`

## Typical Call Flow

1. `motor_controller_new_socketcan`
2. `motor_controller_add_damiao_motor` or `motor_controller_add_robstride_motor`
3. optional: `motor_controller_enable_all`
4. optional: `motor_handle_ensure_mode`
5. send commands / read state / read or write vendor params
6. `motor_controller_shutdown`
7. `motor_handle_free`
8. `motor_controller_free`

## Examples

- C ABI demo: `examples/c/c_abi_demo.c`
- C++ ABI demo: `examples/cpp/cpp_abi_demo.cpp`
- Python ctypes demo: `examples/python/python_ctypes_demo.py`
