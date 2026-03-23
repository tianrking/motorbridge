# Windows Distribution and SDK Usage

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan + Damiao Serial Bridge)

- Linux SocketCAN uses interface names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- Damiao-only serial bridge transport is also available in CLI (`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`).
- On Linux SocketCAN, do not append bitrate in `--channel` (for example `can0@1000000` is invalid).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.

For full cross-platform diagnostics (Linux `slcan` + Windows `pcan`), see [can_debugging.md](can_debugging.md).

This guide explains how to ship and consume `motorbridge` on Windows.

## Artifact Matrix

- Python users:
  - install `.whl` (`motorbridge-<ver>-cp3xx-...-win_amd64.whl`)
- C/C++ users:
  - use ABI `.zip` (`motorbridge-abi-<ver>-windows-x86_64.zip`)
- Linux-only package:
  - `.deb` is for Ubuntu/Linux and is not installable on Windows
- MSI:
  - optional convenience installer, not required for SDK usage

## Runtime Dependencies

- PEAK PCAN driver installed
- `PCANBasic.dll` available from PCAN-Basic runtime

## Build Windows ABI Locally

```bash
cargo build -p motor_abi --release
```

Expected output:

- `target/release/motor_abi.dll`
- `target/release/motor_abi.lib`

## Build Windows Python Wheel Locally

```bash
python -m pip install --user wheel
set MOTORBRIDGE_LIB=%CD%\\target\\release\\motor_abi.dll
python -m pip wheel --no-build-isolation bindings/python -w bindings/python/dist
```

Notes:

- The wheel build now embeds the ABI DLL automatically.
- If the ABI DLL cannot be found, wheel build fails fast with a clear error.

## Install and Verify Python SDK

```bash
python -m pip install bindings/python/dist/motorbridge-*.whl
python -c "from motorbridge import Controller; c=Controller('can0@1000000'); print('ok'); c.close()"
```

## Install and Verify C/C++ ABI

1. Download `motorbridge-abi-<ver>-windows-x86_64.zip`.
2. Extract include/lib to your dependency folder.
3. Link `motor_abi.dll` + import lib in your project.

Minimal ctypes verification:

```python
import ctypes
lib = ctypes.CDLL("motor_abi.dll")
lib.motor_controller_new_socketcan.argtypes = [ctypes.c_char_p]
lib.motor_controller_new_socketcan.restype = ctypes.c_void_p
ptr = lib.motor_controller_new_socketcan(b"can0@1000000")
assert ptr
lib.motor_controller_free(ptr)
```

## Channel and Bitrate Conventions on Windows

- `can0` maps to `PCAN_USBBUS1`
- `can1` maps to `PCAN_USBBUS2`
- Add bitrate as suffix: `can0@1000000`

## Recommended Validation Commands

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4310 --motor-id 0x07 --feedback-id 0x17 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
```
