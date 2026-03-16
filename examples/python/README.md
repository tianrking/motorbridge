# Python Examples

This directory contains Python demos that call Rust `motor_abi` via `ctypes`.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Purpose of This Directory

- Show direct ABI calls from Python (`ctypes`)
- Provide full control-mode command examples (`enable/disable/mit/pos-vel/vel/force-pos`)
- Provide a minimal baseline for debugging ABI issues independent from SDK packaging

If you prefer higher-level usage, see Python SDK CLI in `bindings/python` (`motorbridge-cli`).

## Files

- `python_ctypes_demo.py`: unified multi-mode ctypes demo (`enable/disable/mit/pos-vel/vel/force-pos`)

## Prerequisites

From project root (`rust_dm`):

```bash
cargo build -p motor_abi --release
```

Run from project root so relative `.so` path resolves:

```bash
python3 examples/python/python_ctypes_demo.py --help
```

## Common Parameters

- `--channel`: CAN channel (default `can0`)
- `--model`: motor model (default `4340`)
- `--motor-id`: command ID, e.g. `0x01`
- `--feedback-id`: feedback ID, e.g. `0x11`
- `--mode`: `enable|disable|mit|pos-vel|vel|force-pos`
- `--loop`: send cycles
- `--dt-ms`: interval per cycle (ms)
- `--print-state`: `1/0`, print feedback state
- `--ensure-mode`: `1/0`, ensure control mode before non-enable/disable sends
- `--ensure-timeout-ms`: ensure mode timeout (ms)

Control params:

- MIT: `--pos --vel --kp --kd --tau`
- POS_VEL: `--pos --vlim`
- VEL: `--vel`
- FORCE_POS: `--pos --vlim --ratio`

## Full Commands

Standalone enable:

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100 --print-state 1
```

Standalone disable:

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100 --print-state 1
```

MIT mode:

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0.0 --vel 0.0 --kp 20 --kd 1 --tau 0.0 \
  --ensure-mode 1 --ensure-timeout-ms 1000 \
  --loop 200 --dt-ms 20 --print-state 1
```

POS_VEL mode (reach a target position):

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 \
  --ensure-mode 1 --ensure-timeout-ms 1000 \
  --loop 300 --dt-ms 20 --print-state 1
```

VEL mode:

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 \
  --ensure-mode 1 --ensure-timeout-ms 1000 \
  --loop 100 --dt-ms 20 --print-state 1
```

FORCE_POS mode:

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 \
  --ensure-mode 1 --ensure-timeout-ms 1000 \
  --loop 100 --dt-ms 20 --print-state 1
```

## Related SDK CLI (ID/Scan Tools)

For `id-dump` / `id-set` / `scan`, use `motorbridge-cli` from `bindings/python`:

```bash
motorbridge-cli id-dump --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11
motorbridge-cli id-set --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --new-motor-id 0x02 --new-feedback-id 0x12 --store 1 --verify 1
motorbridge-cli scan --channel can0 --model 4340P --start-id 0x01 --end-id 0x10
```

## Troubleshooting

- `OSError: cannot open shared object file`:
  - build ABI first: `cargo build -p motor_abi --release`
  - run from project root (`rust_dm`)
- `socketcan write failed: Network is down`:
  - bring CAN interface up first (`ip link show can0`)
- repeated `no feedback yet`:
  - verify model, IDs, wiring and power
