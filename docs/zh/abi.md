# ABI 指南 (`motor_abi`)

## 构建

```bash
cargo build -p motor_abi --release
```

产物：

- Linux：`target/release/libmotor_abi.so`、`libmotor_abi.a`
- Windows：`target/release/motor_abi.dll`、`motor_abi.lib`
- 头文件：`motor_abi/include/motor_abi.h`

分发格式：

- Linux x86_64：`.deb` 与 `.tar.gz`
- Windows x86_64：`.zip`（Windows 不能安装 `.deb`）

## 厂商入口

- Damiao: `motor_controller_add_damiao_motor(...)`
- RobStride: `motor_controller_add_robstride_motor(...)`

两个 vendor 共享的公共控制面：

- `motor_handle_enable`
- `motor_handle_disable`
- `motor_handle_send_mit`
- `motor_handle_send_vel`
- `motor_handle_get_state`

Damiao 专属流程：

- `motor_handle_send_pos_vel`
- `motor_handle_send_force_pos`
- Damiao 寄存器读写接口

RobStride 专属 ABI：

- `motor_handle_robstride_ping`
- `motor_handle_robstride_write_param_i8/u8/u16/u32/f32`
- `motor_handle_robstride_get_param_i8/u8/u16/u32/f32`

## 典型调用顺序

1. `motor_controller_new_socketcan`
2. `motor_controller_add_damiao_motor` 或 `motor_controller_add_robstride_motor`
3. 可选：`motor_controller_enable_all`
4. 可选：`motor_handle_ensure_mode`
5. 发控制命令 / 读状态 / 读写厂商参数
6. `motor_controller_shutdown`
7. `motor_handle_free`
8. `motor_controller_free`

## 示例

- C ABI 示例：`examples/c/c_abi_demo.c`
- C++ ABI 示例：`examples/cpp/cpp_abi_demo.cpp`
- Python ctypes 示例：`examples/python/python_ctypes_demo.py`
