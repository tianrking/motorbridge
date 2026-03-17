# motorbridge C++ Bindings (RAII Wrapper)

Thin C++ wrapper on top of the stable C ABI (`motor_abi`).

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Goals

- Keep ABI stability (all calls still go through `motor_abi`)
- Provide modern C++ ergonomics (RAII + exceptions + typed API)
- Keep feature parity with Python SDK core API

## API Parity with Python

`motorbridge::Controller`:

- `enable_all()`, `disable_all()`
- `poll_feedback_once()`
- `shutdown()`, `close_bus()`
- `add_damiao_motor(motor_id, feedback_id, model)`

`motorbridge::Motor`:

- control: `enable()`, `disable()`, `clear_error()`, `set_zero_position()`
- mode: `ensure_mode(mode, timeout_ms)`
- send: `send_mit()`, `send_pos_vel()`, `send_vel()`, `send_force_pos()`
- ops: `request_feedback()`, `set_can_timeout_ms()`, `store_parameters()`
- register: `write_register_f32/u32()`, `get_register_f32/u32()`
- state: `get_state()` (`std::optional<State>`)

Damiao metadata (parity with Python package exports):

- `DAMIAO_RW_REGISTERS`
- `DAMIAO_HIGH_IMPACT_RIDS`
- `DAMIAO_PROTECTION_RIDS`
- `get_damiao_register_spec(rid)`

## Layout

- Header: `include/motorbridge/motorbridge.hpp`
- CMake package: `CMakeLists.txt`, `cmake/motorbridge-config.cmake.in`
- Wrapper demo: `examples/cpp_wrapper_demo.cpp`

## Build Demo Locally

From repo root:

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/cpp_wrapper_demo
```

## Consumer Usage (`find_package`)

After installation/package extraction (contains `include/`, `lib/`, `lib/cmake/motorbridge/`):

```cmake
find_package(motorbridge CONFIG REQUIRED)
add_executable(app main.cpp)
target_link_libraries(app PRIVATE motorbridge::cpp)
```

## Minimal Usage Example

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

