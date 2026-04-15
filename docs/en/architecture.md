# Architecture

## Layered View

```mermaid
flowchart TB
  APP["User Apps (Rust/C/C++/Python/ROS2/WS)"] --> SURFACE["CLI / ABI / SDK / Integrations"]
  SURFACE --> CORE["motor_core (controller, bus, model, traits)"]
  CORE --> DAMIAO["motor_vendors/damiao"]
  CORE --> ROBSTRIDE["motor_vendors/robstride"]
  CORE --> MYACT["motor_vendors/myactuator"]
  CORE --> HIGHTORQUE["motor_vendors/hightorque"]
  CORE --> HEXFELLOW["motor_vendors/hexfellow"]
  CORE --> TEMPLATE["motor_vendors/template (onboarding scaffold)"]
  DAMIAO --> CAN["CAN Bus Backend"]
  ROBSTRIDE --> CAN
  MYACT --> CAN
  HIGHTORQUE --> CAN
  HEXFELLOW --> CAN
  CAN --> LNX["Linux: SocketCAN"]
  CAN --> WIN["Windows (experimental): PEAK PCAN"]
  CAN --> HW["Physical Motors"]
```

## Runtime Control Flow

```mermaid
sequenceDiagram
  participant U as User Tool
  participant C as CoreController
  participant V as Vendor Motor
  participant B as CAN Backend
  U->>C: send command (MIT/POS_VEL/VEL/FORCE_POS)
  C->>V: dispatch by motor handle
  V->>B: encode + transmit frame
  B-->>V: feedback frame
  V-->>C: parsed state
  C-->>U: state cache / readback
```

## Design Goal

`motorbridge` separates generic control infrastructure from vendor-specific protocol logic.

- Core should be reusable across vendors.
- Vendor crates should contain protocol/register/model differences.
- ABI should expose stable cross-language entry points.

## Repository Layout

```text
motorbridge/
├── motor_core/                  # Vendor-agnostic runtime
├── motor_vendors/
│   ├── damiao/                  # Production implementation
│   ├── robstride/               # Production implementation (extended CAN ID / params)
│   ├── myactuator/              # Production implementation (RMD protocol)
│   ├── hightorque/              # Production implementation (ht_can protocol)
│   ├── hexfellow/               # Production implementation (CANopen over CAN-FD)
│   └── template/                # Scaffold for new vendors
├── motor_cli/                   # Rust CLI
├── motor_abi/                   # C ABI (cdylib + staticlib)
├── integrations/
│   ├── ros2_bridge/             # ROS2 bridge
│   └── ws_gateway/              # Rust WebSocket gateway
├── tools/
│   └── factory_calib_ui/        # Factory web UI (calls motor_cli for scan/set-id/verify)
├── bindings/
│   ├── python/                 # Python SDK package + CLI
│   └── cpp/                    # C++ RAII wrapper + CMake package
├── examples/                    # C/C++/Python demo programs
└── docs/
    ├── en/
    └── zh/
```

## Core Layers

### 1) `motor_core`

- `bus.rs`: CAN bus abstraction
- `device.rs`: unified `MotorDevice` trait
- `controller.rs`: scheduling/routing/polling
- `model.rs`: model catalog abstraction
- `socketcan.rs`: Linux classic SocketCAN backend
- `socketcanfd.rs`: Linux dedicated SocketCAN-FD backend
- `pcan.rs`: Windows PEAK PCAN backend (experimental)

### 2) `motor_vendors/*`

Each vendor crate implements:

- frame encode/decode
- register semantics
- motor model limits
- controller facade wrapping `CoreController`

### 3) `motor_abi`

- exports C-compatible handles and functions
- wraps Rust errors into integer return code + `motor_last_error_message()`
- enables C/C++/Python/etc integration

### 4) SDK/Examples

- `motor_cli`: operational/debugging command-line tool
- `bindings/python`: reusable Python package for deployment/integration
- `bindings/cpp`: reusable C++ RAII wrapper for deployment/integration
- `examples/*`: minimal cross-language ABI usage references

## Lifecycle Policy

Controller lifecycle is explicit:

- Use `motor_controller_shutdown` when you want an explicit stop/disable flow.
- Use `motor_controller_close_bus` when you only want to close local session/bus.
- `motor_controller_free` now only releases memory/objects.
