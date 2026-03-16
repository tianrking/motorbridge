# motor_calib

Rust calibration utility for Damiao motors.

## Features

- `scan`: scan CAN bus IDs by register probing
- `set-id`: change `ESC_ID`/`MST_ID` and optionally verify
- `verify`: verify current `ESC_ID`/`MST_ID`

## Build

```bash
cargo build -p motor_calib --release
```

## Usage

```bash
cargo run -p motor_calib -- --help
```

### Scan

```bash
cargo run -p motor_calib -- scan \
  --channel can0 --model 4310 --start-id 0x01 --end-id 0x10 --timeout-ms 100
```

### Set ID

```bash
cargo run -p motor_calib -- set-id \
  --channel can0 --model 4310 \
  --motor-id 0x02 --feedback-id 0x12 \
  --new-motor-id 0x05 --new-feedback-id 0x15 \
  --store 1 --verify 1
```

### Verify

```bash
cargo run -p motor_calib -- verify \
  --channel can0 --model 4310 --motor-id 0x05 --feedback-id 0x15
```
