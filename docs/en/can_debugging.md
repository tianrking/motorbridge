# CAN Debugging Guide (Linux `slcan` + Windows `pcan`)

This guide is the canonical troubleshooting playbook for channel setup and link-level diagnostics.

## 1. Scope and Backend Mapping

- Linux backend: SocketCAN (`can0`, `can1`, `slcan0`, ...)
- Windows backend: PEAK PCAN via `PCANBasic.dll` (`can0/can1` mapping to `PCAN_USBBUS1/2`)

Rules:

- Linux: configure bitrate at interface bring-up time; do not put `@bitrate` in `--channel`.
- Windows PCAN: `@bitrate` suffix is allowed in `--channel` (for example `can0@1000000`).

## 2. Linux `slcan` Bring-up and Verification

### 2.1 Bring up `slcan0`

```bash
sudo pkill slcand 2>/dev/null || true
sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0
sudo ip link set slcan0 up
ip -details link show slcan0
```

Expected:

- Interface exists as `slcan0`
- State is `UP` (or `UNKNOWN`)

### 2.2 Quick traffic sanity checks

Terminal A:

```bash
candump slcan0
```

Terminal B (send a test frame):

```bash
cansend slcan0 001#1122334455667788
```

If no frame appears in `candump`, check adapter wiring, termination, and bitrate consistency with motor firmware.

### 2.3 Run `motor_cli` on `slcan0`

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel slcan0 --mode scan --start-id 1 --end-id 16
```

## 3. Linux Native SocketCAN (`can0`) Quick Checks

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

Counters to watch:

- `RX errors`, `TX errors`, `bus-off`, `re-started`

If `bus-off` grows, fix physical layer first (termination, ground reference, bitrate mismatch).

## 4. Windows PCAN Bring-up and Verification

### 4.1 Preconditions

- Install PEAK driver
- Install PCAN-Basic runtime (`PCANBasic.dll`) and ensure it is loadable
- Confirm adapter channel availability in PEAK tools

### 4.2 Channel conventions used by this project

- `can0` -> `PCAN_USBBUS1`
- `can1` -> `PCAN_USBBUS2`
- Optional bitrate suffix: `can0@1000000`

### 4.3 Validation commands

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16
```

If startup fails with `load PCANBasic.dll failed`, fix PATH/runtime installation first.

## 5. Error-to-Action Map

### Linux SocketCAN path

- `if_nametoindex failed ...`:
  - Interface name is wrong or interface not created/up
  - Action: `ip link show`, recreate `slcan0` or bring up `can0`
- `socketcan write failed` / `socketcan read failed` with hint `interface is down`:
  - Action: `ip -details link show <ifname>`, then bring link up
- `... unavailable` / `interface not found`:
  - Action: check USB adapter presence and interface naming

### Windows PCAN path

- `load PCANBasic.dll failed`:
  - Action: install PCAN-Basic runtime and restart shell/IDE so DLL is discoverable
- `PCAN initialize failed: status=...`:
  - Action: verify channel mapping (`can0/can1`), bitrate suffix, adapter availability
- Repeated reconnect failures:
  - Action: check cable/termination/power and channel occupancy in PEAK tools

## 6. Minimal Cross-Platform Acceptance Checklist

- Linux `can0`: scan command returns expected device IDs
- Linux `slcan0`: same scan command works after `slcand` setup
- Windows `can0@1000000`: scan succeeds with PEAK adapter
- One control command (`mit` or `pos-vel`) succeeds in each environment

If all four pass, channel support for `pcan + slcan` is considered aligned.

