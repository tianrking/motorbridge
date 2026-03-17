# motorbridge C++ 绑定（RAII 封装）

这是基于稳定 C ABI（`motor_abi`）的一层轻量 C++ 封装。

> English version: [README.md](README.md)

## 设计目标

- 保持 ABI 稳定性（底层仍全部调用 `motor_abi`）
- 提供现代 C++ 体验（RAII + 异常 + 强类型接口）
- 与 Python SDK 核心能力对齐

## 与 Python 的能力对齐

`motorbridge::Controller`：

- `enable_all()`、`disable_all()`
- `poll_feedback_once()`
- `shutdown()`、`close_bus()`
- `add_damiao_motor(motor_id, feedback_id, model)`

`motorbridge::Motor`：

- 控制：`enable()`、`disable()`、`clear_error()`、`set_zero_position()`
- 模式：`ensure_mode(mode, timeout_ms)`
- 指令：`send_mit()`、`send_pos_vel()`、`send_vel()`、`send_force_pos()`
- 运维：`request_feedback()`、`set_can_timeout_ms()`、`store_parameters()`
- 寄存器：`write_register_f32/u32()`、`get_register_f32/u32()`
- 状态：`get_state()`（`std::optional<State>`）

Damiao 参数元数据（与 Python 导出对齐）：

- `DAMIAO_RW_REGISTERS`
- `DAMIAO_HIGH_IMPACT_RIDS`
- `DAMIAO_PROTECTION_RIDS`
- `get_damiao_register_spec(rid)`

## 目录结构

- 头文件：`include/motorbridge/motorbridge.hpp`
- CMake 包配置：`CMakeLists.txt`、`cmake/motorbridge-config.cmake.in`
- 封装示例：`examples/cpp_wrapper_demo.cpp`

## 本地构建示例

在仓库根目录执行：

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/cpp_wrapper_demo
```

## 用户侧 `find_package` 用法

在安装/解压后的目录包含 `include/`、`lib/`、`lib/cmake/motorbridge/` 时：

```cmake
find_package(motorbridge CONFIG REQUIRED)
add_executable(app main.cpp)
target_link_libraries(app PRIVATE motorbridge::cpp)
```

## 最小调用示例

```cpp
#include "motorbridge/motorbridge.hpp"

int main() {
  motorbridge::Controller ctrl("can0");
  auto m = ctrl.add_damiao_motor(0x01, 0x11, "4340P");
  ctrl.enable_all();
  m.ensure_mode(motorbridge::Mode::MIT, 1000);
  m.send_mit(0.0f, 0.0f, 20.0f, 1.0f, 0.0f);
  auto st = m.get_state();
  ctrl.shutdown();
  return 0;
}
```

