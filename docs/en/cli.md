# CLI Guide (`motor_cli`)

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan)

- Linux uses SocketCAN channel names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- On Linux, do not append bitrate in `--channel` (for example `can0@1000000` is invalid on SocketCAN).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.

## Debugging Guide

- For deterministic Linux `slcan` + Windows `pcan` troubleshooting, see [can_debugging.md](can_debugging.md).

## Build

```bash
cargo build -p motor_cli --release
```

## Common

- `--vendor damiao|robstride|hightorque|myactuator|all`
- `--channel can0`
- `--motor-id <id>`
- `--loop <n> --dt-ms <ms>`

## Damiao

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

## RobStride

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0 --loop 20 --dt-ms 50
```

## HighTorque (native `ht_can` v1.5.5)

Supported modes:

- `scan`
- `read` / `ping`
- `mit` (unified interface)
- `pos` / `vel` / `tqe`
- `pos-vel-tqe`
- `volt` / `cur`
- `stop` / `brake` / `rezero` / `conf-write` / `timed-read`

Unit interface (aligned with other vendors):

- `--pos` in `rad`
- `--vel` in `rad/s`
- `--tau` in `Nm`
- `--kp`, `--kd` are accepted for unified MIT signature, ignored by `ht_can` protocol

Raw interface (debug):

- `--raw-pos`, `--raw-vel`, `--raw-tqe`

Examples:

```bash
# Scan IDs
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --mode scan --start-id 1 --end-id 32
```

```bash
# Read status (prints pos_rad / vel_rad_s)
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --motor-id 1 --mode read
```

```bash
# Move to +180 deg (pi rad), with velocity/torque limits
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --motor-id 1 \
  --mode mit --pos 3.1415926 --vel 0.8 --tau 0.8
```

```bash
# Stop
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --motor-id 1 --mode stop
```

## MyActuator

```bash
cargo run -p motor_cli --release -- \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode status --loop 20 --dt-ms 50
```

```bash
# Set current position as zero (power-cycle actuator to apply persistently)
cargo run -p motor_cli --release -- \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode set-zero --loop 1
```

## Unified scan

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```
