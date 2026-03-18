# motorbridge C++ Bindings

RAII-style C++ wrapper on top of `motor_abi`.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Controller Entrypoints

- `add_damiao_motor(motor_id, feedback_id, model)`
- `add_robstride_motor(motor_id, feedback_id, model)`

## Quick Start

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

## Example Programs

- `examples/cpp_wrapper_demo.cpp`
- `examples/robstride_wrapper_demo.cpp`
- `examples/full_modes_demo.cpp`
- `examples/pid_register_tune_demo.cpp`
- `examples/scan_ids_demo.cpp` (Damiao legacy helper)
- `examples/pos_ctrl_demo.cpp`
- `examples/pos_repl_demo.cpp`

Unified scan via Rust CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```

## Build

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
```

## End-to-End Demo Commands

```bash
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}

# Damiao wrapper demo
./bindings/cpp/build/cpp_wrapper_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 20 --dt-ms 20

# RobStride wrapper demo: ping
./bindings/cpp/build/robstride_wrapper_demo \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode ping

# RobStride wrapper demo: velocity
./bindings/cpp/build/robstride_wrapper_demo \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```
