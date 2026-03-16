# ABI Usage Guide

## 1. 功能覆盖（Damiao）

当前 ABI 已导出：

- 控制器
  - `motor_controller_new_socketcan`
  - `motor_controller_poll_feedback_once`
  - `motor_controller_enable_all`
  - `motor_controller_disable_all`
  - `motor_controller_shutdown`
  - `motor_controller_close_bus`
  - `motor_controller_free`
- 电机句柄
  - 生命周期: `motor_controller_add_damiao_motor` / `motor_handle_free`
  - 控制: `motor_handle_enable` / `motor_handle_disable` / `motor_handle_clear_error` / `motor_handle_set_zero_position`
  - 模式: `motor_handle_ensure_mode`
  - 指令: `motor_handle_send_mit` / `motor_handle_send_pos_vel` / `motor_handle_send_vel` / `motor_handle_send_force_pos`
  - 寄存器: `motor_handle_write_register_f32` / `motor_handle_write_register_u32` / `motor_handle_get_register_f32` / `motor_handle_get_register_u32`
  - 参数与反馈: `motor_handle_store_parameters` / `motor_handle_request_feedback` / `motor_handle_set_can_timeout_ms` / `motor_handle_get_state`
- 错误
  - `motor_last_error_message`

## 2. 返回值约定

- `0`: 成功
- `-1`: 失败（调用 `motor_last_error_message()` 读错误文本）

## 3. 编译 ABI 库

```bash
cargo build -p motor_abi --release
```

产物：

- `target/release/libmotor_abi.so`
- `target/release/libmotor_abi.a`

头文件：

- `motor_abi/include/motor_abi.h`

### GitHub CI 预构建产物

项目已提供工作流：`.github/workflows/build-abi.yml`，会在 push/PR 时自动构建并上传：

- Linux: `libmotor_abi.so`, `libmotor_abi.a`, `motor_abi.h`
- macOS: `libmotor_abi.dylib`, `libmotor_abi.a`, `motor_abi.h`
- Windows: `motor_abi.dll`, `motor_abi.lib`, `motor_abi.h`

调用方可以直接下载对应平台 artifact，无需本地编译 Rust。

## 4. C/C++ 调用流程

1. 创建控制器 `motor_controller_new_socketcan("can0")`
2. 添加电机 `motor_controller_add_damiao_motor(..., "4340")`
3. `motor_controller_enable_all`
4. `motor_handle_ensure_mode(..., 1, 1000)`（1=MIT）
5. 周期 `motor_handle_send_mit` + `motor_handle_get_state`
6. 结束时 `motor_controller_shutdown` + `motor_handle_free` + `motor_controller_free`

关键点：

- `motor_controller_add_damiao_motor(controller, motor_id, feedback_id, model)` 的 `model` 是可配置字符串，
  不限制为 `4340`。例如可传：`"4340P"`, `"4310"`, `"8006"` 等（需在支持型号列表内）。
- 生命周期建议：
  - 控制结束需要明确失能时：调用 `motor_controller_shutdown`
  - 仅用于扫描/寄存器查询，不想触发失能时：调用 `motor_controller_close_bus` 后 `motor_controller_free`
  - `motor_controller_free` 现在只释放对象，不再隐式调用 `shutdown`

## 5. Python ctypes 调用

见示例：`examples/python/python_ctypes_demo.py`

核心思路：

- `ctypes.CDLL("target/release/libmotor_abi.so")`
- 正确声明 `argtypes/restype`
- 返回值非 0 时读取 `motor_last_error_message()`

示例运行（可选型号）：

```bash
python3 examples/python/python_ctypes_demo.py --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11
```

## 6. C 示例参数化运行

`examples/c/c_abi_demo.c` 已支持命令行参数：

```bash
# 参数: [channel] [model] [motor_id_hex] [feedback_id_hex]
./c_abi_demo can0 4340P 0x01 0x11
```
