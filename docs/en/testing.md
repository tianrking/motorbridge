# Testing Guide

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

## Next Step Improvements

- Add long-run reliability tests for reconnect/error handling
- Add CI matrix stage for Windows runtime smoke (ABI + Python wheel install/import)
