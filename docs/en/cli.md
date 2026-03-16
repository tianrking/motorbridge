# CLI Guide (`motor_cli`)

## Build

```bash
cargo build -p motor_cli --release
```

## Common Args

- `--channel` (default `can0`)
- `--model` (default `4340`)
- `--motor-id` (default `0x01`)
- `--feedback-id` (default `0x11`)
- `--loop` (default `1`)
- `--dt-ms` (default `20`)
- `--ensure-mode` (`1/0`, default `1`)

Model handshake (enabled by default):

- `--verify-model 1/0` (default `1`)
- `--verify-timeout-ms` (default `500`)
- `--verify-tol` (default `0.2`)

## Control Modes

### Enable

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100
```

### Disable

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100
```

### MIT

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 200 --dt-ms 20
```

### POS_VEL

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 300 --dt-ms 20
```

### VEL

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 --loop 100 --dt-ms 20
```

### FORCE_POS

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 --loop 100 --dt-ms 20
```

## Pure Rust ID Update

`motor_cli` supports writing IDs directly:

- `--set-motor-id <id>` => register `rid=8` (`ESC_ID`)
- `--set-feedback-id <id>` => register `rid=7` (`MST_ID`)
- `--store 1/0` => store parameters (default `1`)
- `--verify-id 1/0` => reconnect and verify `rid=8/7` (default `1`)

Example (`0x07/0x17` -> `0x02/0x12`):

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4310 --motor-id 0x07 --feedback-id 0x17 \
  --set-motor-id 0x02 --set-feedback-id 0x12 --store 1 --verify-id 1
```

## ID Scan (Rust-only workflow)

There is no dedicated `scan` subcommand in `motor_cli` yet.
Use a shell probe loop:

```bash
for i in $(seq 1 16); do
  mid=$(printf "0x%02X" "$i")
  fid=$(printf "0x%02X" $((0x10 + (i & 0x0F))))
  if target/release/motor_cli --channel can0 --model 4340P \
      --motor-id "$mid" --feedback-id "$fid" --mode enable --loop 1 --dt-ms 50 \
      >/tmp/mb_scan.log 2>&1; then
    echo "[hit] motor-id=$mid feedback-id=$fid"
  fi
done
```
