# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic
Versioning.

## [Unreleased]

### Added

- Generic architecture split: `motor_core`, vendor crates, and ABI crate.
- Damiao vendor implementation with multi-model support.
- Unified CLI (`motor_cli`) with mode-based parameter control.
- C/C++/Python examples for ABI integration.
- Vendor template scaffold (`motor_vendors/template`) for onboarding new brands.

## [0.1.0] - 2026-03-20

### Added

- Linux USB-CAN (`slcan`) quick guide in root README (EN/ZH), including `slcand` setup and
  `--channel slcan0` usage examples.
- Channel quick reference in `motor_cli/README.md` and `motor_cli/README.zh-CN.md` covering:
  - Linux SocketCAN channels (`can0`, `slcan0`) and Linux rule "no `@bitrate` in channel name"
  - Windows PCAN channel mapping (`can0/can1`) with optional `@bitrate`

### Changed

- CLI startup summary now distinguishes scan semantics from control semantics:
  - `--mode scan` prints `model_hint`, `base_feedback_id`, and `scan_range`
  - defaults are explicitly tagged as `(default)` to reduce confusion

### Fixed

- RobStride frame filtering now only accepts status/fault frames from the target motor ID,
  preventing cross-device state pollution on shared CAN buses.
- Architecture Mermaid diagrams (EN/ZH) now include `myactuator` branch for consistency with
  workspace/runtime layout.

### Usage

- Linux `slcan` setup and examples:
  - `README.md` / `README.zh-CN.md` section: "Linux USB-CAN (`slcan`) Quick Guide"
- Channel compatibility and parameter rules:
  - `motor_cli/README.md` / `motor_cli/README.zh-CN.md` section: "Channel Quick Reference"
