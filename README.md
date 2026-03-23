# motorbridge

Unified CAN motor control stack with a vendor-agnostic Rust core, stable C ABI, and Python/C++ bindings.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Current Vendor Support

- Damiao:
  - models: `3507`, `4310`, `4310P`, `4340`, `4340P`, `6006`, `8006`, `8009`, `10010L`, `10010`, `H3510`, `G6215`, `H6220`, `JH11`, `6248P`
  - modes: `scan`, `MIT`, `POS_VEL`, `VEL`, `FORCE_POS`
- RobStride:
  - models: `rs-00`, `rs-01`, `rs-02`, `rs-03`, `rs-04`, `rs-05`, `rs-06`
  - modes: `scan`, `ping`, `MIT`, `VEL`, parameter read/write
- MyActuator:
  - models: `X8` (runtime string; protocol is ID-based)
  - modes: `scan`, `enable`, `disable`, `stop`, `set-zero`, `status`, `current`, `vel`, `pos`, `version`, `mode-query`
- HighTorque:
  - models: `hightorque` (runtime string; native `ht_can v1.5.5`)
  - modes: `scan`, `read`, `mit`, `pos-vel`, `vel`, `stop`, `brake`, `rezero`

## Architecture

### Layered Runtime View

```mermaid
flowchart TB
  APP["User Apps (Rust/C/C++/Python/ROS2/WS)"] --> SURFACE["CLI / ABI / SDK / Integrations"]
  SURFACE --> CORE["motor_core (controller, bus, model, traits)"]
  CORE --> DAMIAO["motor_vendors/damiao"]
  CORE --> ROBSTRIDE["motor_vendors/robstride"]
  CORE --> MYACT["motor_vendors/myactuator"]
  CORE --> TEMPLATE["motor_vendors/template (onboarding scaffold)"]
  DAMIAO --> CAN["CAN bus backend"]
  ROBSTRIDE --> CAN
  MYACT --> CAN
  CAN --> LNX["Linux: SocketCAN"]
  CAN --> WIN["Windows (experimental): PEAK PCAN"]
  CAN --> HW["Physical motors"]
```

### Workspace Topology (Latest)

```mermaid
flowchart LR
  ROOT["motorbridge workspace"] --> CORE["motor_core"]
  ROOT --> VENDORS["motor_vendors/*"]
  ROOT --> CLI["motor_cli"]
  ROOT --> ABI["motor_abi"]
  ROOT --> TOOLS["tools/motor_calib"]
  ROOT --> INTS["integrations/*"]
  ROOT --> BIND["bindings/*"]
  VENDORS --> VD["damiao"]
  VENDORS --> VR["robstride"]
  VENDORS --> VM["myactuator"]
  VENDORS --> VT["template"]
  INTS --> ROS["ros2_bridge"]
  INTS --> WS["ws_gateway"]
  BIND --> PY["python"]
  BIND --> CPP["cpp"]
```

- [`motor_core`](motor_core): vendor-agnostic controller, routing, CAN bus layer (Linux SocketCAN / Windows experimental PCAN)
- [`motor_vendors/damiao`](motor_vendors/damiao): Damiao protocol / models / registers
- [`motor_vendors/robstride`](motor_vendors/robstride): RobStride extended CAN protocol / models / parameters
- [`motor_vendors/myactuator`](motor_vendors/myactuator): MyActuator CAN protocol implementation
- [`motor_cli`](motor_cli): unified Rust CLI
  - full parameters (English): [`motor_cli/README.md`](motor_cli/README.md)
  - full parameters (Chinese): [`motor_cli/README.zh-CN.md`](motor_cli/README.zh-CN.md)
  - Damiao command/register guide: [`motor_cli/DAMIAO_API.md`](motor_cli/DAMIAO_API.md), [`motor_cli/DAMIAO_API.zh-CN.md`](motor_cli/DAMIAO_API.zh-CN.md)
  - RobStride command/parameter guide: [`motor_cli/ROBSTRIDE_API.md`](motor_cli/ROBSTRIDE_API.md), [`motor_cli/ROBSTRIDE_API.zh-CN.md`](motor_cli/ROBSTRIDE_API.zh-CN.md)
  - MyActuator command/mode guide: [`motor_cli/MYACTUATOR_API.md`](motor_cli/MYACTUATOR_API.md), [`motor_cli/MYACTUATOR_API.zh-CN.md`](motor_cli/MYACTUATOR_API.zh-CN.md)
- [`motor_abi`](motor_abi): stable C ABI
- [`bindings/python`](bindings/python): Python SDK + `motorbridge-cli`
- [`bindings/cpp`](bindings/cpp): C++ RAII wrapper

## Quick Start

Build:

```bash
cargo build
```

Bring up CAN:

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

Quick CAN restart (Linux):

```bash
# default: can0 / 1Mbps / restart-ms=100 / loopback off
IF=can0; BITRATE=1000000; RESTART_MS=100; LOOPBACK=off
sudo ip link set "$IF" down 2>/dev/null || true
if [ "$LOOPBACK" = "on" ]; then
  sudo ip link set "$IF" type can bitrate "$BITRATE" restart-ms "$RESTART_MS" loopback on
else
  sudo ip link set "$IF" type can bitrate "$BITRATE" restart-ms "$RESTART_MS" loopback off
fi
sudo ip link set "$IF" up
ip -details link show "$IF"
```

Damiao CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

HighTorque CLI (native ht_can v1.5.5):

```bash
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --model hightorque --motor-id 1 \
  --mode read
```

RobStride CLI parameter read:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019
```

MyActuator CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode status --loop 20 --dt-ms 50
```

Unified scan (all vendors):

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```

## Experimental Windows Support (PCAN-USB)

Linux remains the primary target. Windows support is experimental and currently backed by PEAK PCAN (`PCANBasic.dll`).

- Install PEAK PCAN driver + PCAN-Basic runtime on Windows.
- Channel mapping:
  - `can0` -> `PCAN_USBBUS1`
  - `can1` -> `PCAN_USBBUS2`
- Optional bitrate suffix: `@<bitrate>` (for example `can0@1000000`).

Validation commands on Windows:

```bash
# Scan Damiao IDs
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16

# Move motor #1 (4340P) to +pi rad (~180 deg)
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20

# Move motor #7 (4310) to +pi rad (~180 deg)
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4310 --motor-id 0x07 --feedback-id 0x17 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
```

## Linux USB-CAN (`slcan`) Quick Guide

Linux uses SocketCAN interface names directly (for example `can0`, `slcan0`).
Do not pass bitrate suffix in Linux channel names (for example `can0@1000000` is invalid on Linux SocketCAN).

Bring up an `slcan` adapter as `slcan0`:

```bash
sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0
sudo ip link set slcan0 up
ip -details link show slcan0
```

Then use `slcan0` as CLI channel:

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel slcan0 --mode scan --start-id 1 --end-id 255
```

## CAN Debugging (Professional Playbook)

For deterministic troubleshooting of Linux `slcan` and Windows `pcan`, use:

- [`docs/en/can_debugging.md`](docs/en/can_debugging.md)
- [`docs/zh/can_debugging.md`](docs/zh/can_debugging.md)

Interpretation:

- `vendor=damiao id=<n>` means one Damiao motor is online at motor ID `<n>`.
- `vendor=robstride id=<n> responder_id=<m>` means one RobStride motor responded.
- `vendor=hightorque ... [hit] id=<n> ...` means one HighTorque motor responded via native ht_can v1.5.5.
- `vendor=myactuator id=<n>` means one MyActuator motor responded.
- `hits=<k>` at the end of each scan block is the count of discovered devices.

## ABI and Bindings

- C ABI:
  - Damiao: `motor_controller_add_damiao_motor(...)`
  - RobStride: `motor_controller_add_robstride_motor(...)`
  - MyActuator: `motor_controller_add_myactuator_motor(...)`
  - HighTorque: `motor_controller_add_hightorque_motor(...)`
- Python:
  - `Controller.add_damiao_motor(...)`
  - `Controller.add_robstride_motor(...)`
  - `Controller.add_myactuator_motor(...)`
  - `Controller.add_hightorque_motor(...)`
- C++:
  - `Controller::add_damiao_motor(...)`
  - `Controller::add_robstride_motor(...)`
  - `Controller::add_myactuator_motor(...)`
  - `Controller::add_hightorque_motor(...)`

Unified mode IDs for ABI/Bindings (`ensure_mode`):

- `1 = MIT`
- `2 = POS_VEL`
- `3 = VEL`
- `4 = FORCE_POS`

Unified control units:

- position: `rad`
- velocity: `rad/s`
- torque: `Nm`

Vendor-specific protocol naming/mapping and unsupported operations are documented in:

- [`docs/en/abi.md`](docs/en/abi.md)
- [`docs/zh/abi.md`](docs/zh/abi.md)

RobStride-specific ABI/binding helpers include:

- `robstride_ping`
- `robstride_get_param_*`
- `robstride_write_param_*`

## Example Entry Points

- Cross-language index: `examples/README.md`
- C ABI demo: `examples/c/c_abi_demo.c`
- C++ ABI demo: `examples/cpp/cpp_abi_demo.cpp`
- Python ctypes demo: `examples/python/python_ctypes_demo.py`
- Python SDK docs: `bindings/python/README.md`
- C++ binding docs: `bindings/cpp/README.md`

## Release Assets Guide

- For C/C++ on Ubuntu x86_64:
  - Download `motorbridge-abi-<tag>-linux-x86_64.deb`
  - Install: `sudo apt install ./motorbridge-abi-<tag>-linux-x86_64.deb`
- For C/C++ on Windows x86_64:
  - Download `motorbridge-abi-<tag>-windows-x86_64.zip`
  - Extract and link `motor_abi.dll/.lib` with bundled headers/CMake config
- For C/C++ on other targets:
  - Use ABI archives (`motorbridge-abi-<tag>-linux-*.tar.gz` or `windows-*.zip`)
  - Link against `libmotor_abi` and use headers/CMake config from the package.
- For Python:
  - Download the matching wheel (`cp310/cp311/cp312` + correct platform/arch)
  - Install: `pip install ./motorbridge-*.whl`
  - Or use source package: `pip install ./motorbridge-*.tar.gz`
- Note:
  - `.deb` is Linux-only; Windows users should use `.zip` and `.whl`.
- Device matrix: `docs/en/devices.md`
