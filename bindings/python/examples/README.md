# Python Practical Demos

Examples built on the Python SDK.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Files

- `python_wrapper_demo.py`: minimal Damiao MIT loop
- `robstride_wrapper_demo.py`: RobStride ping / read-param / mit / vel demo
- `full_modes_demo.py`: Damiao full-mode demo
- `pid_register_tune_demo.py`: Damiao register tuning
- `scan_ids_demo.py`: Damiao fast scan (legacy helper)
- `pos_ctrl_demo.py`: Damiao one-shot target position
- `pos_repl_demo.py`: Damiao interactive position console

## Quick Run

Damiao:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/python_wrapper_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 20 --dt-ms 20
```

RobStride:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride read parameter:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-00 --motor-id 127 --mode read-param --param-id 0x7019
```

Unified vendor scan via CLI:

```bash
PYTHONPATH=bindings/python/src python3 -m motorbridge.cli scan \
  --vendor all --channel can0 --start-id 0x01 --end-id 0xFF
```
