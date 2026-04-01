# motorbridge Python SDK

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan + CAN-FD + Damiao Serial Bridge)

- Linux SocketCAN uses interface names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- CAN-FD transport is available both in CLI (`--transport socketcanfd`) and Python SDK (`Controller.from_socketcanfd(...)`), and is required for Hexfellow.
- Damiao-only serial bridge transport is also available in CLI (`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`).
- Full Damiao serial-bridge interface list and command patterns are documented in `motor_cli/README.md` (section `3.6` in `motor_cli/README.zh-CN.md`).
- On Linux SocketCAN, do not append bitrate in `--channel` (for example `can0@1000000` is invalid).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.


Python binding layer on top of `motor_abi`.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## README Navigation (What Each One Is For)

If this is your first time in this folder, read in this order:

1. This file: [README.md](README.md)  
   Purpose: Python binding overview (install, API scope, common commands).
2. [examples/README.md](examples/README.md) (English) / [examples/READMEzh_cn.md](examples/READMEzh_cn.md) (Chinese)  
   Purpose: practical demo index and run instructions (from simplest to advanced).
2.5. [mintlify/README.md](mintlify/README.md)  
   Purpose: Mintlify documentation site entry for Python binding (tutorial + API style docs).
3. [get_started/README.md](get_started/README.md) / [get_started/README.zh-CN.md](get_started/README.zh-CN.md)  
   Purpose: pip-first onboarding path (install -> scan -> run).
4. [DAMIAO_PYTHON_REFERENCE.zh-CN.md](DAMIAO_PYTHON_REFERENCE.zh-CN.md)  
   Purpose: Damiao Python interface reference (parameter lookup style).
5. [DAMIAO_binding.md](DAMIAO_binding.md)  
   Purpose: Damiao binding implementation notes (design/internal behavior).
6. [README.zh-CN.md](README.zh-CN.md)  
   Purpose: Chinese overview for Chinese-speaking teammates.

Notes:
- If your goal is "run something now", start with `Start Here (Simplest 2 Examples)` in `examples/README.md`.
- If your goal is CLI parameter lookup, see `../../motor_cli/README.md`.

## Scope

- High-level API: `Controller`, `Motor`, `Mode`
- CLI: `motorbridge-cli`
- Controller constructors:
  - `Controller(channel="can0")` (SocketCAN/PCAN path)
  - `Controller.from_socketcanfd(channel="can0")` (CAN-FD path, required by Hexfellow)
  - `Controller.from_dm_serial(serial_port="/dev/ttyACM0", baud=921600)` (Damiao-only serial bridge)
- Vendors:
  - Damiao: `add_damiao_motor(...)`
  - Hexfellow: `add_hexfellow_motor(...)`
  - MyActuator: `add_myactuator_motor(...)`
  - RobStride: `add_robstride_motor(...)`
  - HighTorque: `add_hightorque_motor(...)`
- Unified state-query pattern:
  - Recommended flow: `request_feedback() -> poll_feedback_once() -> get_state()`.
  - RobStride now supports this unified pattern via ABI-level compatibility (while `robstride_ping()` is still available).

## Quick Start

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    motor = ctrl.add_damiao_motor(0x01, 0x11, "4340P")
    ctrl.enable_all()
    motor.ensure_mode(Mode.MIT, 1000)
    motor.send_mit(0.0, 0.0, 20.0, 1.0, 0.0)
    print(motor.get_state())
    motor.close()
```

Damiao over serial bridge:

```python
from motorbridge import Controller, Mode

with Controller.from_dm_serial("/dev/ttyACM1", 921600) as ctrl:
    motor = ctrl.add_damiao_motor(0x04, 0x14, "4310")
    ctrl.enable_all()
    motor.ensure_mode(Mode.MIT, 1000)
    motor.send_mit(0.5, 0.0, 20.0, 1.0, 0.0)
    motor.close()
```

RobStride quick use:

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    motor = ctrl.add_robstride_motor(127, 0xFF, "rs-00")
    print(motor.robstride_ping())
    print(motor.robstride_get_param_f32(0x7019))
    motor.close()
```

MyActuator quick use:

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    motor = ctrl.add_myactuator_motor(1, 0x241, "X8")
    ctrl.enable_all()
    motor.ensure_mode(Mode.POS_VEL, 1000)
    motor.send_pos_vel(3.1416, 2.0)  # rad / rad/s
    print(motor.get_state())
    motor.close()
```

Hexfellow quick use (CAN-FD only):

```python
from motorbridge import Controller, Mode

with Controller.from_socketcanfd("can0") as ctrl:
    motor = ctrl.add_hexfellow_motor(1, 0x00, "hexfellow")
    ctrl.enable_all()
    motor.ensure_mode(Mode.MIT, 1000)      # Hexfellow supports MIT / POS_VEL
    motor.send_mit(0.8, 1.0, 30.0, 1.0, 0.1)
    print(motor.get_state())
    motor.close()
```

## CLI Examples

Damiao:

```bash
motorbridge-cli run \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride:

```bash
motorbridge-cli run \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode ping
```

RobStride parameter read:

```bash
motorbridge-cli robstride-read-param \
  --channel can0 --model rs-00 --motor-id 127 --param-id 0x7019 --type f32
```

Unified scan (all vendors):

```bash
motorbridge-cli scan --vendor all --channel can0 --start-id 0x01 --end-id 0xFF
```

HighTorque via binding:

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    motor = ctrl.add_hightorque_motor(1, 0x01, "hightorque")
    motor.send_mit(3.1416, 0.8, 0.0, 0.0, 0.8)  # kp/kd are accepted but ignored by protocol
    motor.request_feedback()
    print(motor.get_state())
    motor.close()
```

HighTorque via Rust CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --motor-id 1 --mode read
```

## Experimental Windows Support (PCAN-USB)

Linux remains the primary target. Windows support is experimental and currently uses PEAK PCAN.

- Install PEAK PCAN driver + PCAN-Basic runtime (`PCANBasic.dll`).
- Use `channel` as `can0@1000000` (maps to `PCAN_USBBUS1` at 1Mbps).

Recommended quick validation with Rust CLI on Windows:

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4310 --motor-id 0x07 --feedback-id 0x17 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
```

Local wheel build (Windows):

```bash
python -m pip install --user wheel
set MOTORBRIDGE_LIB=%CD%\\target\\release\\motor_abi.dll
python -m pip wheel --no-build-isolation bindings/python -w bindings/python/dist
python -m pip install bindings/python/dist/motorbridge-*.whl
```

## Example Programs

- Damiao wrapper demo: `examples/python_wrapper_demo.py`
- Hexfellow CAN-FD demo: `examples/hexfellow_canfd_demo.py` (MIT / POS_VEL only)
- Damiao maintenance demo: `examples/damiao_maintenance_demo.py`
- Damiao register rw demo: `examples/damiao_register_rw_demo.py`
- Damiao dm-serial demo: `examples/damiao_dm_serial_demo.py`
- RobStride wrapper demo: `examples/robstride_wrapper_demo.py`
- Full Damiao mode demo: `examples/full_modes_demo.py`
- Damiao scan / tune / position helpers:
  - `examples/scan_ids_demo.py`
  - `examples/pid_register_tune_demo.py`
  - `examples/pos_ctrl_demo.py`
  - `examples/pos_repl_demo.py`

See [examples/README.md](examples/README.md) (English) or [examples/READMEzh_cn.md](examples/READMEzh_cn.md) (Chinese).

## Damiao Full-Coverage Status

Damiao usage in Python examples is now covered end-to-end:

- control modes: `mit` / `pos-vel` / `vel` / `force-pos`
- transport paths: SocketCAN/PCAN constructor + `from_dm_serial(...)`
- maintenance ops: `clear_error`, `set_zero_position`, `set_can_timeout_ms`, `request_feedback`
  - project guard for Damiao set-zero: call `disable()` before `set_zero_position()`
  - no user-facing `ms` parameter for set-zero; core applies fixed `20ms` settle
- register APIs: `get/write f32`, `get/write u32`, `store_parameters`

## End-to-End Demo Commands

```bash
# Build ABI once
cargo build -p motor_abi --release
export PYTHONPATH=bindings/python/src
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}

# Damiao wrapper demo
python3 bindings/python/examples/python_wrapper_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 20 --dt-ms 20

# RobStride wrapper demo: ping
python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode ping

# RobStride wrapper demo: velocity
python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

## Notes

- `id-dump` and `id-set` are Damiao workflows; `scan` supports `damiao|hexfellow|myactuator|robstride|hightorque|all`.
- `Mode.MIT` and `send_force_pos` are not available for MyActuator in ABI wrapper.
- Hexfellow supports `MIT` and `POS_VEL` through ABI wrapper; `VEL` and `FORCE_POS` return unsupported.
- Full Damiao tuning reference stays in:
  - [DAMIAO_API.md](DAMIAO_API.md)
  - [DAMIAO_API.zh-CN.md](DAMIAO_API.zh-CN.md)

## PyPI Auto Publish (GitHub Actions)

This repository includes `.github/workflows/pypi-publish.yml`.

- Tag publish policy:
  - push `vX.Y.Z` -> publish the same artifacts to both TestPyPI and PyPI
- Manual publish is still available via workflow `Python Publish`:
  - `testpypi` (only TestPyPI)
  - `pypi` (only PyPI)

### One-time setup (token mode)

1. Create API token on PyPI and add repository secret `PYPI_API_TOKEN`.
2. Create API token on TestPyPI and add repository secret `TEST_PYPI_API_TOKEN`.
3. Keep package version unique for every upload (for example `0.1.6`, `0.1.7`).
