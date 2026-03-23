# Cross-language Examples

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan)

- Linux uses SocketCAN channel names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- On Linux, do not append bitrate in `--channel` (for example `can0@1000000` is invalid on SocketCAN).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.


## Index

- Rust CLI: `motor_cli/src/main.rs`
- C ABI: `examples/c/c_abi_demo.c`
- C++ ABI: `examples/cpp/cpp_abi_demo.cpp`
- Python ctypes: `examples/python/python_ctypes_demo.py`
- Python SDK: `bindings/python/examples/*`
- C++ wrapper: `bindings/cpp/examples/*`

## Quick Commands

```bash
cargo build -p motor_abi --release
```

Damiao Python ctypes:

```bash
python3 examples/python/python_ctypes_demo.py --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

RobStride Python ctypes:

```bash
python3 examples/python/python_ctypes_demo.py --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride C ABI:

```bash
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
LD_LIBRARY_PATH=target/release ./c_abi_demo --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode read-param --param-id 0x7019 --param-type f32
```
