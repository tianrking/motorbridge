# CLI Guide (`motor_cli`)

## Build

```bash
cargo build -p motor_cli --release
```

## Quick CAN Restart (Linux)

```bash
# default: can0 / 1Mbps / restart-ms=100 / loopback off
IF=can0; BITRATE=1000000; RESTART_MS=100; LOOPBACK=off
sudo ip link set "$IF" down 2>/dev/null || true
if [ "$LOOPBACK" = "on" ]; then
  sudo ip link set "$IF" type can bitrate "$BITRATE" restart-ms "$RESTART_MS" loopback on
else
  sudo ip link set "$IF" type can bitrate "$BITRATE" restart-ms "$RESTART_MS" loopback off
fi
sudo ip link set "$IF" up
ip -details link show "$IF"
```

## Common Flag

- `--vendor damiao|robstride|myactuator|all`

## Damiao Examples

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 200 --dt-ms 20
```

## RobStride Examples

Ping:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

Read parameter:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019
```

MIT:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0 --loop 20 --dt-ms 50
```

## MyActuator Examples

Status:

```bash
cargo run -p motor_cli --release -- \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode status --loop 20 --dt-ms 50
```

Velocity:

```bash
cargo run -p motor_cli --release -- \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode vel --vel 0.5236 --loop 100 --dt-ms 20
```

Position:

```bash
cargo run -p motor_cli --release -- \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode pos --pos 3.1416 --max-speed 5.236 --loop 80 --dt-ms 50
```

Unified scan (all vendors):

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```

Output reading:

- Damiao hit line: `vendor=damiao id=<n> ...`
- RobStride hit line: `vendor=robstride id=<n> responder_id=<m> ...`
- MyActuator hit line: `vendor=myactuator id=<n> ...`
- Summary line: `hits=<k>` means discovered online motor count for that vendor.
