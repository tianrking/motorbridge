# motorbridge C++ Bindings

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan + Damiao Serial Bridge)

- Linux SocketCAN uses interface names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- Damiao-only serial bridge transport is also available in CLI (`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`).
- Full Damiao serial-bridge interface list and command patterns are documented in `motor_cli/README.md` (section `3.6` in `motor_cli/README.zh-CN.md`).
- On Linux SocketCAN, do not append bitrate in `--channel` (for example `can0@1000000` is invalid).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.


RAII-style C++ wrapper on top of `motor_abi`.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Damiao Set-Zero Rule (dm-serial)

- For Damiao, call `disable()` before `set_zero_position()`.
- Core guard rejects `set_zero_position()` when motor is not disabled.
- Core applies an internal fixed settle (`~20ms`) after `set_zero_position()`.
- C++/ABI signatures are unchanged; this is behavior guard in core.

## Controller Entrypoints

- `Controller(channel)` (SocketCAN/PCAN path)
- `Controller::from_dm_serial(serial_port, baud)` (Damiao-only serial bridge)
- `add_damiao_motor(motor_id, feedback_id, model)`
- `add_myactuator_motor(motor_id, feedback_id, model)`
- `add_robstride_motor(motor_id, feedback_id, model)`
- `add_hightorque_motor(motor_id, feedback_id, model)`

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

Damiao over serial bridge:

```cpp
#include "motorbridge/motorbridge.hpp"

int main() {
  auto ctrl = motorbridge::Controller::from_dm_serial("/dev/ttyACM1", 921600);
  auto motor = ctrl.add_damiao_motor(0x04, 0x14, "4310");
  ctrl.enable_all();
  motor.send_mit(0.5f, 0.0f, 20.0f, 1.0f, 0.0f);
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

HighTorque via Rust CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --motor-id 1 --mode read
```

HighTorque via C++ binding:

```cpp
#include "motorbridge/motorbridge.hpp"

int main() {
  motorbridge::Controller ctrl("can0");
  auto motor = ctrl.add_hightorque_motor(1, 0x01, "hightorque");
  motor.send_mit(3.1416f, 0.8f, 0.0f, 0.0f, 0.8f);  // kp/kd kept for signature compatibility
  motor.request_feedback();
  auto st = motor.get_state();
  ctrl.shutdown();
  return st.has_value() ? 0 : 1;
}
```

## Experimental Windows Support (PCAN-USB)

Linux remains the primary target. Windows support is experimental and currently uses PEAK PCAN.

- Install PEAK PCAN driver + PCAN-Basic runtime (`PCANBasic.dll`).
- Use `can0@1000000` for channel/bitrate in Windows CLI validation.

Validation commands:

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4310 --motor-id 0x07 --feedback-id 0x17 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
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
