# Python Practical Demos

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan + CAN-FD + Damiao Serial Bridge)

- Linux SocketCAN uses interface names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- Hexfellow examples require CAN-FD path (`Controller.from_socketcanfd(...)` / CLI `--transport socketcanfd`).
- Damiao-only serial bridge transport is also available in CLI (`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`).
- Full Damiao serial-bridge interface list and command patterns are documented in `motor_cli/README.md` (section `3.6` in `motor_cli/README.zh-CN.md`).
- On Linux SocketCAN, do not append bitrate in `--channel` (for example `can0@1000000` is invalid).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.


Examples built on the Python SDK.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Files

- `python_wrapper_demo.py`: minimal Damiao MIT loop
- `damiao_maintenance_demo.py`: Damiao maintenance flow (`clear_error` / `set_zero_position` / `set_can_timeout_ms` / `request_feedback`)
- `damiao_register_rw_demo.py`: Damiao register read/write (`f32` + `u32` + optional `store_parameters`)
- `damiao_dm_serial_demo.py`: Damiao serial-bridge transport demo (`Controller.from_dm_serial`)
- `dm_serial_01_calibration_demo.py`: SOP-01 dm-serial maintenance/calibration flow
- `dm_serial_02_control_modes_demo.py`: SOP-02 dm-serial normal control loop (4 modes, no calibration/config writes)
- `dm_serial_03_status_demo.py`: SOP-03 dm-serial status-only monitor
- `dm_serial_04_enable_setzero_no_delay_demo.py`: SOP-04 dm-serial stress repro (`set_zero` then immediate control)
- `dm_serial_05_setzero_timing_ab_test.py`: SOP-05 dm-serial set-zero settle-time A/B test
- `dm_serial_06_recover_no_reboot_demo.py`: SOP-06 dm-serial software recovery (no host reboot)
- `dm_serial_07_enable_setzero_enable_rotate_demo.py`: SOP-07 dm-serial robust sequence (`disable -> set_zero -> enable -> control`)
- `dm_serial_08_negative_enable_setzero_guard_demo.py`: SOP-08 dm-serial negative test (`enable` state `set_zero` should be rejected by core guard)
- `dm_serial_leader_monitor_demo.py`: Damiao dm-serial leader monitor (enable-all + selected-ID full state stream)
- `robstride_wrapper_demo.py`: RobStride ping / read-param / mit / vel demo
- `hexfellow_canfd_demo.py`: Hexfellow CAN-FD demo (`mit` / `pos-vel` only)
- `full_modes_demo.py`: Damiao full-mode demo
- `pid_register_tune_demo.py`: Damiao register tuning
- `scan_ids_demo.py`: Damiao fast scan (legacy helper)
- `pos_ctrl_demo.py`: Damiao one-shot target position
- `multi_motor_ctrl_demo.py`: Damiao multi-motor control with `-id` / `-pos` one-to-one mapping
- `mit_pos_switch_demo.py`: two-phase mode switch demo (MIT then POS_VEL) for multi-motor targets
- `pos_repl_demo.py`: Damiao interactive position console

## Quick Run

Damiao:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/python_wrapper_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 20 --dt-ms 20
```

Damiao maintenance:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/damiao_maintenance_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --can-timeout-ms 1000 --set-zero 0
```

Damiao register rw:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/damiao_register_rw_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --read-f32-rid 21 --read-u32-rid 10 --store 0
```

Damiao dm-serial:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/damiao_dm_serial_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x01 --feedback-id 0x11 --mode mit --loop 40 --dt-ms 20
```

SOP-01 dm-serial calibration:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_01_calibration_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --set-zero 0 --can-timeout-ms 1000
```

SOP-02 dm-serial normal control (no calibration/config writes):

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_02_control_modes_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --mode pos-vel --pos 0.8 --vlim 1.0 --loop 100 --dt-ms 20
```

SOP-03 dm-serial status monitor:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_03_status_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --loop 100 --dt-ms 50
```

SOP-05 dm-serial set-zero settle-time A/B test:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_05_setzero_timing_ab_test.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --settle-list-ms 0,50,100,200 --rounds 10 --ensure-timeout-ms 500
```

SOP-06 dm-serial software recovery without reboot:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_06_recover_no_reboot_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --attempts 6 --timeout-ms 800
```

SOP-07 dm-serial robust set-zero + control sequence:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_07_enable_setzero_enable_rotate_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --target-pos 3.0 --vlim 1.0 \
  --loop 50 --dt-ms 20 --ensure-timeout-ms 800 \
  --post-setzero-ms 0
```

SOP-08 dm-serial negative test (`enable` then `set_zero`, expect guard reject):

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_08_negative_enable_setzero_guard_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --ensure-timeout-ms 800
```

## dm-serial Timing Notes (Set-Zero Sequence)

- Observed issue: running `set_zero_position()` and immediately calling `ensure_mode(...)` can trigger `register 10 not received` timeouts.
- Root cause pattern: this is typically a dm-serial timing window (bridge latency + motor internal state switch), not a normal control-mode logic bug.
- Core guard rule: `set_zero_position()` is accepted only after `disable()`.
- Core settle rule: after `set_zero_position()`, a fixed `20 ms` settle is applied in core (not exposed as Python argument).
- Recommended sequence for robust control:
  1. `disable` (or `disable_all`)
  2. `set_zero_position`
  3. `enable` (or `enable_all`)
  4. `ensure_mode`
  5. run control loop
- If the timeout state is triggered, try software recovery first (`disable -> clear_error -> enable -> retry`) before rebooting host/device.

Damiao dm-serial leader monitor (selected IDs full-state stream):

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_leader_monitor_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  -id 0x04 0x07 --loop 10000 --dt-ms 20 --hold-mode mit-zero
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

Hexfellow (CAN-FD only):

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/hexfellow_canfd_demo.py \
  --channel can0 --motor-id 0x01 --feedback-id 0x00 --mode mit --loop 20 --dt-ms 50
```

Unified vendor scan via CLI:

```bash
PYTHONPATH=bindings/python/src python3 -m motorbridge.cli scan \
  --vendor all --channel can0 --start-id 0x01 --end-id 0xFF
```

Multi-motor `pos-vel` (example: motor `4` and `7`, positions mapped by order):

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/multi_motor_ctrl_demo.py \
  --channel can0 --model 4310 --mode pos-vel \
  -id 4 7 -pos 0.8 -0.6 -vlim 1.2 1.2 --loop 200 --dt-ms 20
```

MIT/POS_VEL switch demo (POS_VEL-only run for motor `4` and `7`, target `-3`):

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/mit_pos_switch_demo.py \
  --channel can0 --model 4310 -id 4 7 \
  --trajectory -3 \
  --mit-hold-loops 0 --pos-hold-loops 50 \
  --dt-ms 20 --print-state 1
```

## Damiao Coverage Note

Damiao examples now cover the full high-level SDK usage surface:

- Control modes: `mit` / `pos-vel` / `vel` / `force-pos`
- Transport paths: `Controller(channel)` + `Controller.from_dm_serial(...)`
- Maintenance ops: `clear_error`, `set_zero_position`, `set_can_timeout_ms`, `request_feedback`
- Register APIs: `get/write f32`, `get/write u32`, `store_parameters`
- Scan helper and tuning workflows
