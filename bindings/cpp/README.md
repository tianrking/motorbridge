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
- Fast bus scan demo: `examples/scan_ids_demo.cpp`
- One-shot target position demo: `examples/pos_ctrl_demo.cpp`
- Interactive position console demo: `examples/pos_repl_demo.cpp`

## Install Without Rust (Recommended for users)

Users do not need Rust locally. Download prebuilt assets from GitHub Releases.

### 1) Linux x86_64: install `.deb` (recommended)

Download:

- `motorbridge-abi-<version>-linux-x86_64.deb`

Install:

```bash
sudo dpkg -i motorbridge-abi-<version>-linux-x86_64.deb
```

Installed paths:

- headers: `/usr/include/motorbridge/motorbridge.hpp`, `/usr/include/motor_abi.h`
- libs: `/usr/lib/libmotor_abi.so`, `/usr/lib/libmotor_abi.a`
- CMake package: `/usr/lib/cmake/motorbridge/*`

Quick verification:

```bash
dpkg -L motorbridge-abi | grep -E "motorbridge.hpp|motor_abi.h|libmotor_abi|cmake/motorbridge"
```

### 2) Linux aarch64 / Windows x86_64: use release archive

Download and extract:

- Linux aarch64: `motorbridge-abi-<version>-linux-aarch64.tar.gz`
- Windows x86_64: `motorbridge-abi-<version>-windows-x86_64.zip`

After extraction, keep this structure:

- `<prefix>/include/motorbridge/motorbridge.hpp`
- `<prefix>/include/motor_abi.h`
- `<prefix>/lib/*` (contains `libmotor_abi.so` or `motor_abi.dll/.lib`)
- `<prefix>/lib/cmake/motorbridge/*`

Then pass `<prefix>` to `CMAKE_PREFIX_PATH`.

## Consumer Usage (`find_package`)

`CMakeLists.txt`:

```cmake
find_package(motorbridge CONFIG REQUIRED)
add_executable(app main.cpp)
target_link_libraries(app PRIVATE motorbridge::cpp)
```

Configure:

- Linux `.deb` install:

```bash
cmake -S . -B build
cmake --build build -j
```

- Manual extracted package:

```bash
cmake -S . -B build -DCMAKE_PREFIX_PATH=/path/to/motorbridge-prefix
cmake --build build -j
```

Minimal compile/link verification (after `.deb` install):

```bash
cat > CMakeLists.txt <<'CMAKE'
cmake_minimum_required(VERSION 3.16)
project(mb_check LANGUAGES CXX)
find_package(motorbridge CONFIG REQUIRED)
add_executable(mb_check main.cpp)
target_link_libraries(mb_check PRIVATE motorbridge::cpp)
CMAKE
cat > main.cpp <<'CPP'
#include "motorbridge/motorbridge.hpp"
int main() { return motorbridge::get_damiao_register_spec(motorbridge::RID_CTRL_MODE) ? 0 : 1; }
CPP
cmake -S . -B build && cmake --build build -j && ./build/mb_check
```

Runtime library notes:

- Linux: make sure loader can find `libmotor_abi.so` (`ldconfig`, `LD_LIBRARY_PATH`, or rpath).
- Windows: keep `motor_abi.dll` next to your `.exe` (or add its folder to `PATH`).

## CAN setup before running (Linux)

If you see `socketcan write failed: Network is down (os error 100)`, `can0` is not up.

Recommended commands (1Mbps example):

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

Optional bus monitor (`can-utils`):

```bash
candump can0
```

Install if needed:

```bash
sudo apt install can-utils
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

## Build Demo Locally (for developers)

From repo root:

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/cpp_wrapper_demo
```

## Practical C++ Demos

Build:

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
```

1) Fast bus scan:

```bash
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/scan_ids_demo \
  --channel can0 --model 4310 --start-id 0x01 --end-id 0xFF --feedback-base 0x10 --timeout-ms 120
```

2) One-shot target position (POS_VEL):

```bash
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/pos_ctrl_demo \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --target-pos 3.14 --vlim 1.5 --loop 300 --dt-ms 20
```

3) Interactive position console:

```bash
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/pos_repl_demo \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 --vlim 1.5
```

Then type values like `1` or `3.14` and press Enter.
