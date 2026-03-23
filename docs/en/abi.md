# ABI Guide (`motor_abi`)

## Build

```bash
cargo build -p motor_abi --release
```

Artifacts:

- Linux: `target/release/libmotor_abi.so`, `libmotor_abi.a`
- Windows: `target/release/motor_abi.dll`, `motor_abi.lib`
- Header: `motor_abi/include/motor_abi.h`

## Unified Surface

The ABI keeps one unified control surface across vendors.

Unified mode IDs (`motor_handle_ensure_mode`):

- `1 = MIT`
- `2 = POS_VEL`
- `3 = VEL`
- `4 = FORCE_POS`

Unified control units:

- position: `rad`
- velocity: `rad/s`
- torque: `Nm`

Common control/state APIs:

- `motor_handle_enable`
- `motor_handle_disable`
- `motor_handle_clear_error`
- `motor_handle_set_zero_position`
- `motor_handle_ensure_mode`
- `motor_handle_send_mit`
- `motor_handle_send_pos_vel`
- `motor_handle_send_vel`
- `motor_handle_send_force_pos`
- `motor_handle_request_feedback`
- `motor_handle_set_can_timeout_ms`
- `motor_handle_store_parameters`
- `motor_handle_get_state`

## Vendor Entry Points

- Damiao: `motor_controller_add_damiao_motor(...)`
- RobStride: `motor_controller_add_robstride_motor(...)`
- MyActuator: `motor_controller_add_myactuator_motor(...)`
- HighTorque: `motor_controller_add_hightorque_motor(...)`

## Unified Mode to Native Protocol Mapping

| Unified mode | Damiao native | RobStride native | MyActuator native | HighTorque native |
|---|---|---|---|---|
| `MIT` | `Mit` | `Mit` | not available | mapped to native pos+vel+tqe |
| `POS_VEL` | `PosVel` | not available | `Position` setpoint flow | mapped to native pos+vel+tqe |
| `VEL` | `Vel` | `Velocity` | `Velocity` setpoint flow | mapped to native velocity command |
| `FORCE_POS` | `ForcePos` | not available | not available | not available |

Behavior rule:

- Unsupported calls return non-zero and a readable message via `motor_last_error_message()`.
- Signatures stay stable even when a vendor ignores part of a signature.
- Example: HighTorque accepts `send_mit(pos, vel, kp, kd, tau)` for interface consistency, but native protocol does not use `kp/kd`.

## Vendor-Specific Extensions

Damiao register APIs:

- `motor_handle_write_register_f32`
- `motor_handle_write_register_u32`
- `motor_handle_get_register_f32`
- `motor_handle_get_register_u32`

RobStride extensions:

- `motor_handle_robstride_ping`
- `motor_handle_robstride_set_device_id`
- `motor_handle_robstride_write_param_i8/u8/u16/u32/f32`
- `motor_handle_robstride_get_param_i8/u8/u16/u32/f32`

## Typical Call Flow

1. Transport constructor:
   - `motor_controller_new_socketcan(channel)` (general path)
   - `motor_controller_new_dm_serial(serial_port, baud)` (Damiao-only serial bridge; unix-like systems)
2. `motor_controller_add_<vendor>_motor`
3. optional: `motor_controller_enable_all`
4. optional: `motor_handle_ensure_mode`
5. send control commands / read state / vendor-specific operations
6. `motor_controller_shutdown`
7. `motor_handle_free`
8. `motor_controller_free`

## Examples

- C ABI demo: `examples/c/c_abi_demo.c`
- C++ ABI demo: `examples/cpp/cpp_abi_demo.cpp`
- Python ctypes demo: `examples/python/python_ctypes_demo.py`
