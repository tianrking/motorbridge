# Examples Index

This directory helps you quickly find:

- Full parameter sets per control mode
- Method mapping by language (CLI / Python / C / C++)
- Python SDK subcommands for ID tooling (`id-dump` / `id-set` / `scan`)
- Direct command examples, including standalone `enable/disable`

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## File Index

- Rust CLI: `motor_cli/src/main.rs`
- Rust native examples:
  - `motor_vendors/damiao/examples/test_4340.rs`
  - `motor_vendors/damiao/examples/test_4340p.rs`
- C ABI demo: `examples/c/c_abi_demo.c`
- C examples README (EN): `examples/c/README.md`
- C examples README (ZH): `examples/c/README.zh-CN.md`
- C++ ABI demo: `examples/cpp/cpp_abi_demo.cpp`
- C++ examples README (EN): `examples/cpp/README.md`
- C++ examples README (ZH): `examples/cpp/README.zh-CN.md`
- C++ wrapper package (recommended for C++ integration): `bindings/cpp`
- Python ctypes demo: `examples/python/python_ctypes_demo.py`
- Python examples README (EN): `examples/python/README.md`
- Python examples README (ZH): `examples/python/README.zh-CN.md`
- Damiao API/tuning full reference (EN): `examples/dm_api.md`
- Damiao API/tuning full reference (ZH): `examples/dm_api_cn.md`
- Python SDK package (recommended for integration): `bindings/python`
- ABI header: `motor_abi/include/motor_abi.h`

## Common Device Parameters

| Param | Description | Default |
|---|---|---|
| `channel` | CAN interface name | `can0` |
| `model` | Motor model (`4340`, `4340P`, `4310`, etc.) | `4340` |
| `motor-id` | Command ID (`ESC_ID`) | `0x01` |
| `feedback-id` | Feedback ID (`MST_ID`) | `0x11` |
| `loop` | Send cycles | `1` |
| `dt-ms` | Send period in ms | `20` |
| `ensure-mode` | Ensure control mode before sending (`1/0`) | `1` |
| `verify-model` | Verify `--model` by reading `PMAX/VMAX/TMAX` (`1/0`) | `1` |
| `verify-timeout-ms` | Register read timeout for model verification | `500` |
| `verify-tol` | Absolute tolerance for `PMAX/VMAX/TMAX` comparison | `0.2` |

## CAN Setup First (Required)

Before running any example command:

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

## Full Parameters per Mode (CLI)

### 1) Standalone Enable

- Mode: `--mode enable`
- Optional: `--loop`, `--dt-ms`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 1
```

### 2) Standalone Disable

- Mode: `--mode disable`
- Optional: `--loop`, `--dt-ms`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 1
```

### Model Handshake Verification (recommended)

- Default behavior: enabled (`--verify-model 1`)
- Read registers: `rid=21/22/23` (`PMAX/VMAX/TMAX`)
- If mismatch: CLI exits and prints suggested models

Pass case (correct model):

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 5 --dt-ms 100
```

Mismatch case (intentional wrong model):

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 5 --dt-ms 100
```

### 3) MIT Mode

- Mode: `--mode mit`
- Required control params: `--pos --vel --kp --kd --tau`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 30 --kd 1 --tau 0 --loop 200 --dt-ms 20
```

### 4) POS_VEL Mode (Position-Velocity)

- Mode: `--mode pos-vel`
- Required control params: `--pos --vlim`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 1.2 --vlim 2.0 --loop 100 --dt-ms 20
```

Target-position trial (reach `3.10 rad` with velocity limit):

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 300 --dt-ms 20
```

### 5) VEL Mode

- Mode: `--mode vel`
- Required control params: `--vel`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 --loop 100 --dt-ms 20
```

### 6) FORCE_POS Mode

- Mode: `--mode force-pos`
- Required control params: `--pos --vlim --ratio`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 --loop 100 --dt-ms 20
```

## Method Mapping (CLI / Python / C / C++)

| Action | CLI | Python ctypes / C / C++ ABI |
|---|---|---|
| Enable | `--mode enable` | `motor_handle_enable(...)` |
| Disable | `--mode disable` | `motor_handle_disable(...)` |
| MIT send | `--mode mit ...` | `motor_handle_send_mit(...)` |
| POS_VEL send | `--mode pos-vel ...` | `motor_handle_send_pos_vel(...)` |
| VEL send | `--mode vel ...` | `motor_handle_send_vel(...)` |
| FORCE_POS send | `--mode force-pos ...` | `motor_handle_send_force_pos(...)` |
| Ensure mode | `--ensure-mode 1` | `motor_handle_ensure_mode(...)` |
| Dump IDs/mode/limits | `motorbridge-cli id-dump ...` | `motor_handle_get_register_u32/f32(...)` |
| Set IDs | `motorbridge-cli id-set ...` | `motor_handle_write_register_u32(...)` + `motor_handle_store_parameters(...)` |
| Scan IDs | `motorbridge-cli scan ...` | register probe loop with ABI reads |

Mode values for ABI:

- `1=MIT`, `2=POS_VEL`, `3=VEL`, `4=FORCE_POS`

## Cross-language Quick Run

### Python ctypes

```bash
cargo build -p motor_abi --release
python3 examples/python/python_ctypes_demo.py --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11
```

### C

```bash
cargo build -p motor_abi --release
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
LD_LIBRARY_PATH=target/release ./c_abi_demo can0 4340P 0x01 0x11
```

### C++

```bash
cargo build -p motor_abi --release
g++ -std=c++17 examples/cpp/cpp_abi_demo.cpp -I motor_abi/include -L target/release -lmotor_abi -o cpp_abi_demo
LD_LIBRARY_PATH=target/release ./cpp_abi_demo --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

## Useful Runtime APIs

- `motor_controller_poll_feedback_once(...)`
- `motor_handle_get_state(...)`
- `motor_handle_write_register_f32/u32(...)`
- `motor_handle_get_register_f32/u32(...)`
- `motor_handle_clear_error(...)`
- `motor_handle_set_zero_position(...)`
- `motor_handle_store_parameters(...)`
- `motor_handle_set_can_timeout_ms(...)`
- `motor_controller_close_bus(...)` (close session without implicit shutdown)

## Important Behavior Notes

- In `motor_cli`, `--mode enable` and `--mode disable` now close only the local bus/session on exit.
- They no longer trigger implicit auto-disable via shutdown.
