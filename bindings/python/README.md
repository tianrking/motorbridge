# motorbridge Python SDK

Python binding layer on top of `motor_abi`.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Scope

- High-level API: `Controller`, `Motor`, `Mode`
- CLI: `motorbridge-cli`
- Vendors:
  - Damiao: `add_damiao_motor(...)`
  - MyActuator: `add_myactuator_motor(...)`
  - RobStride: `add_robstride_motor(...)`

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
- RobStride wrapper demo: `examples/robstride_wrapper_demo.py`
- Full Damiao mode demo: `examples/full_modes_demo.py`
- Damiao scan / tune / position helpers:
  - `examples/scan_ids_demo.py`
  - `examples/pid_register_tune_demo.py`
  - `examples/pos_ctrl_demo.py`
  - `examples/pos_repl_demo.py`

See [examples/README.md](examples/README.md).

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

- `id-dump` and `id-set` are Damiao workflows; `scan` supports `damiao|myactuator|robstride|all`.
- `Mode.MIT` and `send_force_pos` are not available for MyActuator in ABI wrapper.
- Full Damiao tuning reference stays in:
  - [DAMIAO_API.md](DAMIAO_API.md)
  - [DAMIAO_API.zh-CN.md](DAMIAO_API.zh-CN.md)
