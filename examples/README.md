# Examples Index

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan + Damiao Serial Bridge)

- Linux SocketCAN uses interface names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- Damiao-only serial bridge transport is also available in CLI (`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`).
- Full Damiao serial-bridge interface list and command patterns are documented in `motor_cli/README.md` (section `3.6` in `motor_cli/README.zh-CN.md`).
- On Linux SocketCAN, do not append bitrate in `--channel` (for example `can0@1000000` is invalid).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.


Cross-language example entry for the current `motorbridge` stack.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Coverage

- Rust CLI: `motor_cli/src/main.rs`
- C ABI demo: `examples/c/c_abi_demo.c`
- C++ ABI demo: `examples/cpp/cpp_abi_demo.cpp`
- Python ctypes demo: `examples/python/python_ctypes_demo.py`
- Python SDK demos: `bindings/python/examples/*`
- C++ wrapper demos: `bindings/cpp/examples/*`
- Damiao tuning reference:
  - `../motor_cli/DAMIAO_API.md`
  - `../motor_cli/DAMIAO_API.zh-CN.md`
- RobStride API/parameter reference:
  - `../motor_cli/ROBSTRIDE_API.md`
  - `../motor_cli/ROBSTRIDE_API.zh-CN.md`
- MyActuator command/mode reference:
  - `../motor_cli/MYACTUATOR_API.md`
  - `../motor_cli/MYACTUATOR_API.zh-CN.md`

## Vendor Support in Examples

- Damiao:
  - modes: `enable`, `disable`, `mit`, `pos-vel`, `vel`, `force-pos`
  - register / ID workflows remain available through CLI, Python SDK, and calibration tools
- RobStride:
  - modes: `ping`, `enable`, `disable`, `mit`, `vel`, `read-param`, `write-param`
  - parameter examples use the RobStride ABI and binding helpers
- MyActuator:
  - modes: `scan`, `enable`, `disable`, `stop`, `status`, `current`, `vel`, `pos`, `version`, `mode-query`
  - CLI input uses radians/rad-s for `pos`/`vel` (`motor_cli` converts to protocol degrees internally)

## CAN Setup

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

## Experimental Windows Support (PCAN-USB)

Linux remains the primary target. Windows support is experimental and currently uses PEAK PCAN.

- Install PEAK PCAN driver + PCAN-Basic runtime (`PCANBasic.dll`).
- Use `can0@1000000` as the Windows channel form.

Windows quick validation commands:

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4310 --motor-id 0x07 --feedback-id 0x17 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
```

## Quick Start

Damiao with Rust CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride with Rust CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode ping
```

RobStride parameter read:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019
```

MyActuator with Rust CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode pos --pos 3.1416 --max-speed 5.236 --loop 1 --dt-ms 50
```

## Cross-language ABI Demos

Python ctypes:

```bash
cargo build -p motor_abi --release
python3 examples/python/python_ctypes_demo.py --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
python3 examples/python/python_ctypes_demo.py --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

C:

```bash
cargo build -p motor_abi --release
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
LD_LIBRARY_PATH=target/release ./c_abi_demo --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
LD_LIBRARY_PATH=target/release ./c_abi_demo --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

C++:

```bash
cargo build -p motor_abi --release
g++ -std=c++17 examples/cpp/cpp_abi_demo.cpp -I motor_abi/include -L target/release -lmotor_abi -o cpp_abi_demo
LD_LIBRARY_PATH=target/release ./cpp_abi_demo --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
LD_LIBRARY_PATH=target/release ./cpp_abi_demo --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

## Recommended Higher-level Examples

- Python SDK:
  - `bindings/python/examples/python_wrapper_demo.py`
  - `bindings/python/examples/robstride_wrapper_demo.py`
- C++ wrapper:
  - `bindings/cpp/examples/cpp_wrapper_demo.cpp`
  - `bindings/cpp/examples/robstride_wrapper_demo.cpp`

## Validation Checklist (suggested order)

1. Scan all vendors on the same bus.
2. Verify Damiao control path (MIT or velocity).
3. Verify RobStride control path (ping/read-param/velocity).
4. Verify MyActuator control path (position or velocity).
5. Verify Python binding demos (Damiao + RobStride).
6. Verify C++ binding demos (Damiao + RobStride).

Quick commands:

```bash
# 1) Unified scan
cargo run -p motor_cli --release -- --vendor all --channel can0 --mode scan --start-id 1 --end-id 255

# 2) Damiao quick velocity
cargo run -p motor_cli --release -- --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode vel --vel 0.5 --loop 40 --dt-ms 50

# 3) RobStride ping + velocity
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode ping
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode vel --vel 0.3 --loop 40 --dt-ms 50

# 4) MyActuator position (radians)
cargo run -p motor_cli --release -- --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 --mode pos --pos 3.1416 --max-speed 5.236 --loop 1 --dt-ms 50
```

## Notes

- `id-dump` and `id-set` are Damiao-oriented workflows; unified `scan` is available in Rust CLI (`--vendor all`) and Python SDK CLI (`motorbridge.cli scan --vendor all`).
- RobStride examples focus on ping, parameter access, MIT, and velocity control.
