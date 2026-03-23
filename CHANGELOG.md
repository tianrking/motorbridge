# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic
Versioning.

## [Unreleased]

## [0.1.2] - 2026-03-23

### Changed

- Release version bump from `0.1.1` to `0.1.2` for clean tag progression.
- Damiao `dm-serial` documentation rollout remains aligned across:
  - CLI README (full interface section)
  - root README
  - bindings/examples/integrations/tools related READMEs.

## [0.1.1] - 2026-03-23

### Added

- Damiao serial-bridge transport (`dm-serial`) for unix-like systems:
  - CLI transport selection: `--transport auto|socketcan|dm-serial`
  - Serial options: `--serial-port`, `--serial-baud`
  - Damiao controller serial constructor and transport runtime wiring.
- C ABI constructor for Damiao serial bridge:
  - `motor_controller_new_dm_serial(serial_port, baud)`
- SDK support for Damiao serial bridge:
  - Python: `Controller.from_dm_serial(...)`
  - C++: `Controller::from_dm_serial(...)`
- New Chinese operation manual for deployment/runtime usage:
  - `docs/zh/operation_manual.md`

### Changed

- README alignment across examples/bindings/integrations/tools:
  - All Damiao-related READMEs now mention `dm-serial` availability.
  - Added explicit pointer to complete interface/command section in
    `motor_cli/README.zh-CN.md` (`3.6`) and `motor_cli/README.md`.

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
