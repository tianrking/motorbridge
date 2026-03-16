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

## Architecture

```text
motorbridge/
├── motor_core/              # Generic core (vendor-agnostic)
│   ├── bus.rs               # CAN bus abstraction
│   ├── device.rs            # Unified MotorDevice trait
│   ├── controller.rs        # CoreController scheduler + routing
│   ├── model.rs             # Model catalog abstractions
│   └── socketcan.rs         # Linux SocketCAN backend
├── motor_vendor_damiao/     # Damiao plugin (protocol/registers/models)
├── motor_vendor_template/   # Template for adding new vendors
├── motor_cli/               # Unified CLI for mode/parameter control
├── motor_abi/               # C ABI (cdylib/staticlib)
├── docs/
│   ├── SUPPORTED_DEVICES.md
│   ├── ABI_USAGE.md
│   ├── EXTENDING.md
│   └── DAMIAO_PARITY.md
└── examples/
    └── README.md            # Cross-language examples index
```

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
```

Build ABI only:

```bash
cargo build -p motor_abi --release
```

ABI outputs:

- `target/release/libmotor_abi.so`
- `target/release/libmotor_abi.a`

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

## ABI and Cross-language Usage

- ABI guide: [docs/ABI_USAGE.md](docs/ABI_USAGE.md)
- C example: [examples/c/c_abi_demo.c](examples/c/c_abi_demo.c)
- C++ example: [examples/cpp/cpp_abi_demo.cpp](examples/cpp/cpp_abi_demo.cpp)
- Python ctypes example: [examples/python/python_ctypes_demo.py](examples/python/python_ctypes_demo.py)
- Full examples index: [examples/README.md](examples/README.md)

## Adding New Vendors

Use [motor_vendor_template](motor_vendor_template) as the scaffold.

Detailed guide: [docs/EXTENDING.md](docs/EXTENDING.md)

## Notes

- Linux SocketCAN backend is currently implemented.
- For each specific motor model/firmware, hardware regression is still recommended.

## Community

- Contributing: [CONTRIBUTING.md](CONTRIBUTING.md)
- Code of Conduct: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- Security Policy: [SECURITY.md](SECURITY.md)
- Changelog: [CHANGELOG.md](CHANGELOG.md)

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).
