# Testing Guide

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan)

- Linux uses SocketCAN channel names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- On Linux, do not append bitrate in `--channel` (for example `can0@1000000` is invalid on SocketCAN).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.


This project currently focuses on deterministic unit tests for protocol and parsing logic, plus workspace-level compilation checks.

## What Is Covered

- `motor_core`:
  - Windows PCAN channel/bitrate parsing and validation
  - `CoreController` integration tests with fake `CanBus`:
    - duplicate device-id rejection
    - frame routing
    - enable/disable fan-out
    - shutdown lifecycle behavior
- `motor_vendor_damiao`:
  - protocol encode/decode primitives
  - model matching/suggestion logic
- `motor_vendor_robstride`:
  - extended CAN ID build/parse
  - ping/parameter encoding and validation
- `motor_cli`:
  - input parsing helpers and RobStride parameter value parsing

## Run All Tests

```bash
cargo test --workspace --all-targets
```

## Recommended Local Quality Gate

```bash
cargo check --workspace
cargo test --workspace --all-targets
```

## Hardware-in-the-loop (manual)

Automated tests avoid real CAN hardware. For hardware validation, run:

1. vendor scan
2. enable/disable
3. control mode command
4. feedback/state readback

Use the commands in root `README.md` (Linux) and Windows experimental section (`can0@1000000`) for repeatable checks.

Reliability helper scripts:

- [`tools/reliability/README.md`](../../tools/reliability/README.md)
- `tools/reliability/reliability_runner.py`

## Next Step Improvements

- Expand long-run HIL matrix (different adapters and bus loads)
- Add periodic cross-platform compare-scan jobs with explicit tolerance policy
