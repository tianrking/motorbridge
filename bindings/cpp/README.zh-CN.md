# motorbridge C++ 绑定

这是基于 `motor_abi` 的 RAII 风格 C++ 包装层。

> English version: [README.md](README.md)

## Controller 入口

- `add_damiao_motor(motor_id, feedback_id, model)`
- `add_myactuator_motor(motor_id, feedback_id, model)`
- `add_robstride_motor(motor_id, feedback_id, model)`
- `add_hightorque_motor(motor_id, feedback_id, model)`

## 快速开始

Damiao:

```cpp
#include "motorbridge/motorbridge.hpp"

int main() {
  motorbridge::Controller ctrl("can0");
  auto motor = ctrl.add_damiao_motor(0x01, 0x11, "4340P");
  ctrl.enable_all();
  motor.ensure_mode(motorbridge::Mode::MIT, 1000);
  motor.send_mit(0.0f, 0.0f, 20.0f, 1.0f, 0.0f);
  ctrl.shutdown();
  return 0;
}
```

RobStride:

```cpp
#include "motorbridge/motorbridge.hpp"

int main() {
  motorbridge::Controller ctrl("can0");
  auto motor = ctrl.add_robstride_motor(127, 0xFF, "rs-00");
  auto ids = motor.robstride_ping();
  float pos = motor.robstride_get_param_f32(0x7019);
  ctrl.shutdown();
  return static_cast<int>(ids.first == 127 && pos > -1000.0f);
}
```

MyActuator:

```cpp
#include "motorbridge/motorbridge.hpp"

int main() {
  motorbridge::Controller ctrl("can0");
  auto motor = ctrl.add_myactuator_motor(1, 0x241, "X8");
  ctrl.enable_all();
  motor.ensure_mode(motorbridge::Mode::POS_VEL, 1000);
  motor.send_pos_vel(3.1416f, 2.0f);  // rad / rad/s
  ctrl.shutdown();
  return 0;
}
```

## 示例程序

- `examples/cpp_wrapper_demo.cpp`
- `examples/robstride_wrapper_demo.cpp`
- `examples/full_modes_demo.cpp`
- `examples/pid_register_tune_demo.cpp`
- `examples/scan_ids_demo.cpp`（Damiao 历史辅助）
- `examples/pos_ctrl_demo.cpp`
- `examples/pos_repl_demo.cpp`

通过 Rust CLI 统一扫描:

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```

通过 Rust CLI 使用 HighTorque：

```bash
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --motor-id 1 --mode read
```

通过 C++ 绑定使用 HighTorque：

```cpp
#include "motorbridge/motorbridge.hpp"

int main() {
  motorbridge::Controller ctrl("can0");
  auto motor = ctrl.add_hightorque_motor(1, 0x01, "hightorque");
  motor.send_mit(3.1416f, 0.8f, 0.0f, 0.0f, 0.8f);  // kp/kd 保留用于统一签名
  motor.request_feedback();
  auto st = motor.get_state();
  ctrl.shutdown();
  return st.has_value() ? 0 : 1;
}
```

## Windows 实验支持（PCAN-USB）

项目主线仍以 Linux 为主。Windows 支持为实验性能力，当前通过 PEAK PCAN 后端实现。

- 安装 PEAK 驱动与 PCAN-Basic 运行时（`PCANBasic.dll`）。
- Windows 下验证命令建议使用 `can0@1000000`。

验证命令：

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4310 --motor-id 0x07 --feedback-id 0x17 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
```

## 构建

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
```

## 端到端示例命令

```bash
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}

# Damiao wrapper 示例
./bindings/cpp/build/cpp_wrapper_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 20 --dt-ms 20

# RobStride wrapper 示例：ping
./bindings/cpp/build/robstride_wrapper_demo \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode ping

# RobStride wrapper 示例：速度
./bindings/cpp/build/robstride_wrapper_demo \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```
