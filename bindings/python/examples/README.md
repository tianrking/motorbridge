# Python Practical Demos

Practical Python demos aligned with `bindings/cpp/examples`:

- `python_wrapper_demo.py`: MIT loop demo (minimal)
- `full_modes_demo.py`: unified full-parameter mode demo (`enable/disable/mit/pos-vel/vel/force-pos`)
- `pid_register_tune_demo.py`: PID/high-impact register tuning + readback verify
- `scan_ids_demo.py`: fast bus scan
- `pos_ctrl_demo.py`: one-shot target position (POS_VEL)
- `pos_repl_demo.py`: interactive position console

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## CAN Setup First

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

## Quick Run

From repo root:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/python_wrapper_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

## Full-Mode Commands (All Params)

Enable:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100 --print-state 1
```

Disable:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100 --print-state 1
```

MIT (`pos/vel/kp/kd/tau`):

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 200 --dt-ms 20 --print-state 1
```

POS_VEL (`pos/vlim`):

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 300 --dt-ms 20 --print-state 1
```

VEL (`vel`):

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 100 --dt-ms 20 --print-state 1
```

FORCE_POS (`pos/vlim/ratio`):

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 100 --dt-ms 20 --print-state 1
```

## PID and High-Impact Register Tuning

Common high-impact registers:

- PID: `KP_ASR(25)`, `KI_ASR(26)`, `KP_APR(27)`, `KI_APR(28)`
- Mapping/safety dynamics: `PMAX(21)`, `VMAX(22)`, `TMAX(23)`, `ACC(4)`, `DEC(5)`, `MAX_SPD(6)`

Example:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/pid_register_tune_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --kp-asr 20 --ki-asr 0.2 --kp-apr 30 --ki-apr 0.1 \
  --pmax 12.5 --vmax 45 --tmax 18 --acc 30 --dec -30 --max-spd 30 \
  --store 1 --timeout-ms 1000
```

## Scan and Position Helpers

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/scan_ids_demo.py \
  --channel can0 --model 4310 --start-id 0x01 --end-id 0xFF --feedback-base 0x10 --timeout-ms 120
```

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/pos_ctrl_demo.py \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --target-pos 3.14 --vlim 1.5 --loop 300 --dt-ms 20
```

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/pos_repl_demo.py \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 --vlim 1.5
```
