# motorbridge

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/tianrking/motorbridge/actions/workflows/ci.yml/badge.svg)](https://github.com/tianrking/motorbridge/actions/workflows/ci.yml)

A unified, high-reliability motor control stack for CAN-based actuators.

Repository: https://github.com/tianrking/motorbridge.git

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Why motorbridge

`motorbridge` is designed to decouple **generic motor control infrastructure** from **vendor-specific protocol implementations**.

- One core runtime for bus I/O, scheduling, and multi-device routing
- Vendor plugins for protocol/register/model differences
- A stable C ABI for C/C++/Python/other languages
- Clear path to add more brands without rewriting core logic

## Tech Stack

- Core language: **Rust** (edition 2021)
- Low-level bus backend: **Linux SocketCAN** (FFI/system calls)
- Cross-language integration: **C ABI** (`cdylib` + `staticlib`)
- Runtime consumers (examples included): Rust / C / C++ / Python (`ctypes`)
- Runtime consumers (via C ABI extension): Go / C# / Java (JNI/JNA) / Node.js (`ffi-napi`) / others

## Architecture

```text
motorbridge/
├── motor_core/              # Generic core (vendor-agnostic)
│   ├── bus.rs               # CAN bus abstraction
│   ├── device.rs            # Unified MotorDevice trait
│   ├── controller.rs        # CoreController scheduler + routing
│   ├── model.rs             # Model catalog abstractions
│   └── socketcan.rs         # Linux SocketCAN backend
├── motor_vendors/
│   ├── damiao/              # Damiao plugin (protocol/registers/models)
│   └── template/            # Template for adding new vendors
├── motor_cli/               # Unified CLI for mode/parameter control
├── motor_abi/               # C ABI (cdylib/staticlib)
├── integrations/
│   ├── ros2_bridge/         # ROS2 bridge (implemented)
│   └── ws_gateway/          # WebSocket gateway (implemented, V1)
├── tools/
│   └── motor_calib/         # Calibration tool (scan / set-id / verify)
├── bindings/
│   └── python/              # Python SDK package (pip / motorbridge-cli)
├── docs/
│   ├── en/                    # English docs
│   └── zh/                    # Chinese docs
└── examples/
    └── README.md            # Cross-language examples index
```

`motor_vendors/` is the canonical vendor namespace directory.  
Each subdirectory represents one vendor implementation (for example `damiao`) or one onboarding scaffold (`template`).

## Current Support

See [docs/en/devices.md](docs/en/devices.md).

Production support today:

- Vendor: **Damiao**
- Models: `3507`, `4310`, `4310P`, `4340`, `4340P`, `6006`, `8006`, `8009`, `10010L`, `10010`, `H3510`, `G6215`, `H6220`, `JH11`, `6248P`
- Modes: `MIT`, `POS_VEL`, `VEL`, `FORCE_POS`

## Capability Matrix

### Core control capabilities

| Capability | `motor_cli` | `motor_calib` | Python SDK CLI | Python API | C ABI |
|---|---|---|---|---|---|
| Enable / Disable | Yes | No | Yes (`run --mode enable/disable`) | Yes | Yes |
| MIT command | Yes | No | Yes | Yes | Yes |
| POS_VEL command | Yes | No | Yes | Yes | Yes |
| VEL command | Yes | No | Yes | Yes | Yes |
| FORCE_POS command | Yes | No | Yes | Yes | Yes |
| Ensure control mode | Yes | No | Yes | Yes | Yes |
| Read motor state | Yes | No | Yes | Yes | Yes |
| Model handshake verify (`PMAX/VMAX/TMAX`) | Yes | No | No | Optional (manual) | Optional (manual) |

### Configuration / calibration capabilities

| Capability | `motor_cli` | `motor_calib` | Python SDK CLI | Python API | C ABI |
|---|---|---|---|---|---|
| Scan active IDs | Script loop | Yes (`scan`) | Yes (`scan`) | Yes (custom loop) | Yes (custom loop) |
| Set `ESC_ID` / `MST_ID` | Yes (`--set-*`) | Yes (`set-id`) | Yes (`id-set`) | Yes | Yes |
| Verify IDs by register read | Yes (`--verify-id`) | Yes (`verify`) | Yes (`id-dump`) | Yes | Yes |
| Read key registers (`7/8/9/10/21/22/23`) | No (dedicated cmd not yet) | Via `verify` | Yes (`id-dump`) | Yes | Yes |
| Write register (`u32`/`f32`) | Limited built-ins | No | Via API | Yes | Yes |
| Store parameters (`0xAA`) | Yes (ID flow) | Yes | Yes | Yes | Yes |

### Why this is strong

- One protocol core, many control surfaces (Rust CLI/tooling + Python + C ABI).
- Fast operation tools (`motor_calib`) and integration APIs can coexist.
- Same hardware capability can be reached from multiple language/runtime paths.

## `motor_cli` vs `motor_calib`

They are intentionally separated, not duplicated:

- `motor_cli`: primary online control tool for enable/disable, MIT, motion commands, mode switching, and model handshake checks.
- `motor_calib`: calibration/operations tool for bus scanning, ID assignment, and ID verification.

Recommended usage:

- Daily control and integration testing: use `motor_cli`.
- Device addressing, replacement, and maintenance: use `motor_calib`.
- Single-tool workflow is possible (`motor_cli` can set IDs), but for batch maintenance, `motor_calib` is the better fit.

### `motor_cli` examples

Show help:

```bash
cargo run -p motor_cli -- --help
```

1. Standalone enable

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100
```

2. Standalone disable

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100
```

3. MIT control

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 200 --dt-ms 20
```

4. POS_VEL control (reach target position)

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 300 --dt-ms 20
```

5. VEL control

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 --loop 100 --dt-ms 20
```

6. FORCE_POS control

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 --loop 100 --dt-ms 20
```

7. Set IDs directly in `motor_cli` (pure Rust path)

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4310 --motor-id 0x07 --feedback-id 0x17 \
  --set-motor-id 0x02 --set-feedback-id 0x12 --store 1 --verify-id 1
```

### `motor_calib` examples

Show help:

```bash
cargo run -p motor_calib -- --help
```

1. Scan online IDs

```bash
cargo run -p motor_calib -- scan \
  --channel can0 --model 4310 --start-id 0x01 --end-id 0x10 --timeout-ms 100
```

2. Set IDs (standard calibration flow)

```bash
cargo run -p motor_calib -- set-id \
  --channel can0 --model 4310 \
  --motor-id 0x02 --feedback-id 0x12 \
  --new-motor-id 0x05 --new-feedback-id 0x15 \
  --store 1 --verify 1
```

3. Verify IDs

```bash
cargo run -p motor_calib -- verify \
  --channel can0 --model 4310 --motor-id 0x05 --feedback-id 0x15
```

## Build

```bash
cargo check
cargo build --release
```

Build only core + Damiao plugin:

```bash
cargo build -p motor_core -p motor_vendor_damiao --release
# note: crate motor_vendor_damiao is located at motor_vendors/damiao
```

Build ABI only:

```bash
cargo build -p motor_abi --release
```

ABI outputs:

- `target/release/libmotor_abi.so`
- `target/release/libmotor_abi.a`

GitHub CI prebuilt ABI artifacts:

- Workflow: `.github/workflows/build-abi.yml`
- On each push/PR, CI uploads ABI artifacts for:
  `linux-x86_64`, `linux-aarch64`, `windows-x86_64`
- On version tags (`v*.*.*`), CI also publishes Python wheels to GitHub Releases:
  Linux (`x86_64` / `aarch64`), Windows (`x86_64`)
- Download from GitHub Actions artifacts or GitHub Releases, then use ABI/SDK examples directly

## Quick Start (CLI)

Show CLI help:

```bash
cargo run -p motor_cli -- --help
```

MIT example:

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 30 --kd 1 --tau 0 --loop 200 --dt-ms 20
```

POS_VEL example:

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 1.2 --vlim 2.0 --loop 100 --dt-ms 20
```

POS_VEL target-position command (reach and hold near a target position):

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 300 --dt-ms 20
```

Model handshake verification (enabled by default):

- CLI reads `PMAX/VMAX/TMAX` (`rid=21/22/23`) on startup.
- If `--model` does not match device limits, CLI exits with mismatch error and suggested models.
- Disable only when intentionally bypassing: `--verify-model 0`.

Quick verification tests:

```bash
# 1) Expected pass (correct model)
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 5 --dt-ms 100

# 2) Expected fail (intentional wrong model, should show suggestions)
cargo run -p motor_cli --release -- \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 5 --dt-ms 100
```

## ABI and Cross-language Usage

- Docs index (EN): [docs/en/index.md](docs/en/index.md)
- Docs index (ZH): [docs/zh/index.md](docs/zh/index.md)
- ABI guide (EN): [docs/en/abi.md](docs/en/abi.md)
- CLI guide (EN): [docs/en/cli.md](docs/en/cli.md)
- Integrations index: [integrations/README.md](integrations/README.md)
- ROS2 bridge (EN): [integrations/ros2_bridge/README.md](integrations/ros2_bridge/README.md)
- WS gateway (EN): [integrations/ws_gateway/README.md](integrations/ws_gateway/README.md)
- Calibration tool (EN): [tools/motor_calib/README.md](tools/motor_calib/README.md)
- C example: [examples/c/c_abi_demo.c](examples/c/c_abi_demo.c)
- C++ example: [examples/cpp/cpp_abi_demo.cpp](examples/cpp/cpp_abi_demo.cpp)
- Python ctypes example: [examples/python/python_ctypes_demo.py](examples/python/python_ctypes_demo.py)
- Python SDK package: [bindings/python](bindings/python)
- Python SDK CLI subcommands: `run` / `id-dump` / `id-set` / `scan`
- Full examples index (EN): [examples/README.md](examples/README.md)
- Full examples index (ZH): [examples/README.zh-CN.md](examples/README.zh-CN.md)

## Adding New Vendors

Use [motor_vendors/template](motor_vendors/template) as the scaffold.

Detailed guide: [docs/en/extending.md](docs/en/extending.md)

## Notes

- Linux SocketCAN backend is currently implemented.
- For each specific motor model/firmware, hardware regression is still recommended.
- `motor_cli` `enable/disable` mode now exits by closing only the local bus session (no implicit auto-disable on exit).

## Community

- Contributing: [CONTRIBUTING.md](CONTRIBUTING.md)
- Code of Conduct: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- Security Policy: [SECURITY.md](SECURITY.md)
- Changelog: [CHANGELOG.md](CHANGELOG.md)

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).
