# ABI 指南（`motor_abi`）

## 构建

```bash
cargo build -p motor_abi --release
```

产物：

- Linux：`target/release/libmotor_abi.so`、`libmotor_abi.a`
- Windows：`target/release/motor_abi.dll`、`motor_abi.lib`
- 头文件：`motor_abi/include/motor_abi.h`

## 统一接口面

ABI 对外保持一套统一控制接口。

统一模式 ID（`motor_handle_ensure_mode`）：

- `1 = MIT`
- `2 = POS_VEL`
- `3 = VEL`
- `4 = FORCE_POS`

统一控制单位：

- 位置：`rad`
- 速度：`rad/s`
- 力矩：`Nm`

通用控制/状态接口：

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

## 厂商入口

- Damiao：`motor_controller_add_damiao_motor(...)`
- RobStride：`motor_controller_add_robstride_motor(...)`
- MyActuator：`motor_controller_add_myactuator_motor(...)`
- HighTorque：`motor_controller_add_hightorque_motor(...)`

## 统一模式与厂商原生协议映射

| 统一模式 | Damiao 原生 | RobStride 原生 | MyActuator 原生 | HighTorque 原生 |
|---|---|---|---|---|
| `MIT` | `Mit` | `Mit` | 不支持 | 映射到原生 pos+vel+tqe |
| `POS_VEL` | `PosVel` | 不支持 | `Position` 设定流程 | 映射到原生 pos+vel+tqe |
| `VEL` | `Vel` | `Velocity` | `Velocity` 设定流程 | 映射到原生速度命令 |
| `FORCE_POS` | `ForcePos` | 不支持 | 不支持 | 不支持 |

行为约定：

- 不支持的调用返回非 0，并可通过 `motor_last_error_message()` 获取清晰错误信息。
- 即使某厂商忽略部分参数，也保持统一函数签名不变。
- 例如：HighTorque 支持 `send_mit(pos, vel, kp, kd, tau)` 统一签名，但原生协议不使用 `kp/kd`。
- Damiao 置零顺序规则：先调用 `motor_handle_disable`，再调用 `motor_handle_set_zero_position`；否则会被核心防护拒绝。
- Damiao 置零稳定规则：`set_zero_position` 成功后，核心层内置固定稳定等待（约 `20ms`），ABI 不额外暴露等待参数。

## 厂商扩展接口

Damiao 寄存器接口：

- `motor_handle_write_register_f32`
- `motor_handle_write_register_u32`
- `motor_handle_get_register_f32`
- `motor_handle_get_register_u32`

RobStride 扩展接口：

- `motor_handle_robstride_ping`
- `motor_handle_robstride_set_device_id`
- `motor_handle_robstride_write_param_i8/u8/u16/u32/f32`
- `motor_handle_robstride_get_param_i8/u8/u16/u32/f32`

## 典型调用顺序

1. 选择传输层构造器：
   - `motor_controller_new_socketcan(channel)`（通用路径）
   - `motor_controller_new_dm_serial(serial_port, baud)`（仅 Damiao 串口桥；跨平台，可用 `/dev/ttyACM0` 或 `COM3`）
2. `motor_controller_add_<vendor>_motor`
3. 可选：`motor_controller_enable_all`
4. 可选：`motor_handle_ensure_mode`
5. 发送控制命令 / 读取状态 / 调用厂商扩展接口
6. `motor_controller_shutdown`
7. `motor_handle_free`
8. `motor_controller_free`

## 示例

- C ABI 示例：`examples/c/c_abi_demo.c`
- C++ ABI 示例：`examples/cpp/cpp_abi_demo.cpp`
- Python ctypes 示例：`examples/python/python_ctypes_demo.py`
