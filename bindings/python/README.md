# motorbridge Python SDK

Python package for calling `motorbridge` Rust ABI.

> 中文版: [README.zh-CN.md](README.zh-CN.md)

## Install

### A) Install from release wheel (recommended for users)

```bash
pip install motorbridge-0.1.0-<python_tag>-linux_x86_64.whl
```

Example:

```bash
pip install motorbridge-0.1.0-cp310-cp310-linux_x86_64.whl
```

### B) Local editable install (for development)

From repo root:

```bash
cd bindings/python
pip install -e .
```

Before runtime, build Rust ABI once (for local dev path):

```bash
cd ../../
cargo build -p motor_abi --release
```

## Python API Quick Use

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    m = ctrl.add_damiao_motor(0x01, 0x11, "4340P")
    ctrl.enable_all()
    m.ensure_mode(Mode.MIT, 1000)
    m.send_mit(0.0, 0.0, 20.0, 1.0, 0.0)
    print(m.get_state())
    m.close()
```

## CLI

```bash
motorbridge-cli --help
```

## CLI Parameters

Common:

- `--channel` (default `can0`)
- `--model` (default `4340`)
- `--motor-id` (default `0x01`)
- `--feedback-id` (default `0x11`)
- `--mode` (`enable|disable|mit|pos-vel|vel|force-pos`)
- `--loop` (default `100`)
- `--dt-ms` (default `20`)
- `--print-state` (`1/0`, default `1`)
- `--ensure-mode` (`1/0`, default `1`, non-enable/disable only)
- `--ensure-timeout-ms` (default `1000`)
- `--ensure-strict` (`1/0`, default `0`)

Mode params:

- MIT: `--pos --vel --kp --kd --tau`
- POS_VEL: `--pos --vlim`
- VEL: `--vel`
- FORCE_POS: `--pos --vlim --ratio`

## Full CLI Commands

Enable:

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100 --print-state 1
```

Disable:

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100 --print-state 1
```

MIT:

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 200 --dt-ms 20 --print-state 1
```

POS_VEL:

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 300 --dt-ms 20 --print-state 1
```

VEL:

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 100 --dt-ms 20 --print-state 1
```

FORCE_POS:

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 100 --dt-ms 20 --print-state 1
```

## Shared Library Resolution

Priority order:

1. `MOTORBRIDGE_LIB` environment variable
2. packaged `motorbridge/lib/*`
3. repo `target/release/*`
4. `ctypes.util.find_library("motor_abi")`

## Troubleshooting

- `Failed to load motor_abi shared library`:
  - ensure wheel includes `motorbridge/lib/libmotor_abi.so`
  - or export `MOTORBRIDGE_LIB=/path/to/libmotor_abi.so`
- `socketcan write failed: Network is down`:
  - bring up CAN interface first (`ip link show can0`)
- repeated `no feedback yet`:
  - verify `feedback-id`, CAN wiring and power
