# Python Binding Quickstart (pip-first)

This folder is a fresh, beginner-first quickstart for users who install from pip.

Goal: install package -> run scan -> run a motor example in minutes.

## 1) Install

```bash
python3 -m pip install motorbridge
```

If you want to test pre-release package from TestPyPI:

```bash
python3 -m pip install -i https://test.pypi.org/simple/ motorbridge==<version>
```

## 2) Hardware and Channel

- Linux SocketCAN channel examples: `can0`, `can1`, `slcan0`
- Windows PCAN channel examples: `can0@1000000`, `can1@1000000`
- Ensure only one sender is writing to the bus while testing.

### Which transport should I use?

- `TRANSPORT = "auto"` or `"socketcan"`:
  use normal CAN path (`CHANNEL` is used).
- `TRANSPORT = "dm-serial"`:
  use Damiao serial bridge (`SERIAL_PORT` / `SERIAL_BAUD` are used), Damiao only.

## 3) Quick Commands (no source checkout required)

```bash
# Unified scan from installed CLI
motorbridge-cli scan --vendor all --channel can0 --start-id 1 --end-id 255

# Single Damiao control (example IDs)
motorbridge-cli run --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 1.0 --vlim 1.0 --loop 60 --dt-ms 20
```

## 4) Quick Python Examples in this folder

- `quickstart_01_scan.py`: run scan in pure Python package path (no subprocess)
- `quickstart_02_single_motor.py`: single motor control via Python SDK (default Damiao)
- `quickstart_03_quad_vendor.py`: 4-motor mixed-vendor example using separate controllers

### Config constants (simple meaning)

- `TRANSPORT`: `auto/socketcan/dm-serial`
- `CHANNEL`: CAN interface name (`can0`, `slcan0`, `can0@1000000`, ...)
- `VENDOR`: scan target vendor (`all` is most common)
- `MOTOR_ID` / `FEEDBACK_ID`: motor command/feedback IDs
- `MODEL`: motor model string
- `TARGET_POS` / `POS`: target position in radians
- `V_LIMIT`: velocity limit for position mode
- `LOOP`: send loop count
- `DT_MS`: control period in ms
- `SERIAL_PORT` / `SERIAL_BAUD`: used only for `dm-serial`

## 5) Course Series (workflow-first)

If you want a strict real-world workflow course, use `courses/`:

- `courses/00-enable-and-status.py`
- `courses/01-scan.py`
- `courses/02-register-rw.py`
- `courses/03-mode-switch-method.py`
- `courses/04-mode-mit.py`
- `courses/05-mode-pos-vel.py`
- `courses/06-mode-vel.py`
- `courses/07-mode-force-pos.py`
- `courses/08-mode-mixed-switch.py`
- `courses/09-multi-motor.py`

Example:

```bash
python3 bindings/python/get_started/courses/00-enable-and-status.py
python3 bindings/python/get_started/courses/01-scan.py
python3 bindings/python/get_started/courses/03-mode-switch-method.py
```

## 6) Run Quickstart Examples

```bash
python3 bindings/python/get_started/quickstart_01_scan.py --channel can0 --vendor all
python3 bindings/python/get_started/quickstart_02_single_motor.py --channel can0 --loop 80 --dt-ms 20
python3 bindings/python/get_started/quickstart_03_quad_vendor.py --channel can0 --loop 120 --dt-ms 20
```

## 7) Common Issues

- `os error 105`: bus TX is too fast or another process is sending; increase `--dt-ms` to 30/50.
- no motor response: verify CAN wiring, bitrate, motor/feedback IDs.
- `slcan` users: bring up `slcan0` before running examples.

## 8) What to read next

- Python API overview: `bindings/python/README.md`
- Full examples catalog: `bindings/python/examples/README.md`
- CLI full docs: `motor_cli/README.md`
