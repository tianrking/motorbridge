# motor_calib

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan + Damiao Serial Bridge)

- Linux SocketCAN uses interface names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- Damiao-only serial bridge transport is also available in CLI (`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`).
- On Linux SocketCAN, do not append bitrate in `--channel` (for example `can0@1000000` is invalid).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.


Rust calibration utility for Damiao motors.

```mermaid
flowchart LR
  SCAN["scan"] --> HIT["find active IDs"]
  HIT --> SET["set-id"]
  SET --> VERIFY["verify"]
  VERIFY --> DONE["persisted address map"]
```

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

CAN setup required before scanning/readdressing:

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
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

## Experimental Windows Support (PCAN-USB)

Linux remains the primary target. Windows support is experimental and currently uses PEAK PCAN.

- Install PEAK PCAN driver + PCAN-Basic runtime (`PCANBasic.dll`).
- Use `can0@1000000` as channel on Windows for 1Mbps tests.

Windows quick validation with `motor_cli`:

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4310 --motor-id 0x07 --feedback-id 0x17 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
```
