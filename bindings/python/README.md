# motorbridge Python SDK

Python package for calling `motorbridge` Rust ABI.

> 中文版: [README.zh-CN.md](README.zh-CN.md)

## Install

### A) Install from release wheel (recommended)

```bash
pip install motorbridge-0.1.0-<python_tag>-<platform>.whl
```

Example:

```bash
pip install motorbridge-0.1.0-cp310-cp310-linux_x86_64.whl
```

### B) Local editable install (development)

```bash
cd bindings/python
pip install -e .
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

## Complete Damiao Tuning Reference

For full Damiao parameter interfaces (control modes, register meaning, tunable range, and tuning workflow):

- [DAMIAO_API.md](DAMIAO_API.md) (English)
- [DAMIAO_API.zh-CN.md](DAMIAO_API.zh-CN.md) (Chinese)
- [../examples/dm_api.md](../examples/dm_api.md) (full table + CLI examples)
- [../examples/dm_api_cn.md](../examples/dm_api_cn.md) (完整参数表 + CLI 示例)

## CLI Overview

`motorbridge-cli` now supports subcommands:

- `run`: control loop (default; legacy flat flags still work)
- `id-dump`: read key registers (`7,8,9,10,21,22,23` by default)
- `id-set`: set `ESC_ID`/`MST_ID` (register `8`/`7`) and optional verify
- `scan`: probe a CAN ID range by register read

Help:

```bash
motorbridge-cli --help
motorbridge-cli run --help
motorbridge-cli id-dump --help
motorbridge-cli id-set --help
motorbridge-cli scan --help
```

## `run` Command Parameters

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

Examples:

```bash
# standalone enable
motorbridge-cli run \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100 --print-state 1

# standalone disable
motorbridge-cli run \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100 --print-state 1

# MIT
motorbridge-cli run \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 200 --dt-ms 20 --print-state 1

# POS_VEL (target position)
motorbridge-cli run \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 300 --dt-ms 20 --print-state 1
```

Legacy compatibility (still supported):

```bash
motorbridge-cli --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

## ID/Scan Commands

Dump key registers:

```bash
motorbridge-cli id-dump \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --timeout-ms 500
```

Set IDs (write `ESC_ID`/`MST_ID`):

```bash
motorbridge-cli id-set \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --new-motor-id 0x02 --new-feedback-id 0x12 \
  --store 1 --verify 1 --timeout-ms 800
```

Scan ID range:

```bash
motorbridge-cli scan \
  --channel can0 --model 4340P \
  --start-id 0x01 --end-id 0x10 --feedback-base 0x10 --timeout-ms 80
```

## Shared Library Resolution

Priority order:

1. `MOTORBRIDGE_LIB` environment variable
2. packaged `motorbridge/lib/*`
3. repo `target/release/*`
4. `ctypes.util.find_library("motor_abi")`

## Troubleshooting

- `Failed to load motor_abi shared library`:
  - ensure wheel includes `motorbridge/lib/libmotor_abi.so` (or platform equivalent)
  - or export `MOTORBRIDGE_LIB=/path/to/libmotor_abi.so`
- `socketcan write failed: Network is down`:
  - bring CAN interface up first (`ip link show can0`)
- repeated `no feedback yet`:
  - verify model, `motor-id`, `feedback-id`, CAN wiring and power
