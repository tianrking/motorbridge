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
├── bindings/
│   └── python/              # Python SDK package (pip / motorbridge-cli)
├── docs/
│   ├── SUPPORTED_DEVICES.md
│   ├── ABI_USAGE.md
│   ├── EXTENDING.md
│   └── DAMIAO_PARITY.md
└── examples/
    └── README.md            # Cross-language examples index
```

`motor_vendors/` is the canonical vendor namespace directory.  
Each subdirectory represents one vendor implementation (for example `damiao`) or one onboarding scaffold (`template`).

## Current Support

See [docs/SUPPORTED_DEVICES.md](docs/SUPPORTED_DEVICES.md).

Production support today:

- Vendor: **Damiao**
- Models: `3507`, `4310`, `4310P`, `4340`, `4340P`, `6006`, `8006`, `8009`, `10010L`, `10010`, `H3510`, `G6215`, `H6220`, `JH11`, `6248P`
- Modes: `MIT`, `POS_VEL`, `VEL`, `FORCE_POS`

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
- On each push/PR, CI uploads platform artifacts (`linux` / `macos` / `windows`)
- Download from GitHub Actions artifacts, then use the ABI examples directly

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

- ABI guide: [docs/ABI_USAGE.md](docs/ABI_USAGE.md)
- C example: [examples/c/c_abi_demo.c](examples/c/c_abi_demo.c)
- C++ example: [examples/cpp/cpp_abi_demo.cpp](examples/cpp/cpp_abi_demo.cpp)
- Python ctypes example: [examples/python/python_ctypes_demo.py](examples/python/python_ctypes_demo.py)
- Python SDK package: [bindings/python](bindings/python)
- Python SDK CLI subcommands: `run` / `id-dump` / `id-set` / `scan`
- Full examples index (EN): [examples/README.md](examples/README.md)
- Full examples index (ZH): [examples/README.zh-CN.md](examples/README.zh-CN.md)

## Adding New Vendors

Use [motor_vendors/template](motor_vendors/template) as the scaffold.

Detailed guide: [docs/EXTENDING.md](docs/EXTENDING.md)

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
