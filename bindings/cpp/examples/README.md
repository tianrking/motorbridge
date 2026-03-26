# C++ Example Programs

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan + Damiao Serial Bridge)

- Linux SocketCAN uses interface names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- Damiao-only serial bridge transport is also available in CLI (`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`).
- Full Damiao serial-bridge interface list and command patterns are documented in `motor_cli/README.md` (section `3.6` in `motor_cli/README.zh-CN.md`).
- On Linux SocketCAN, do not append bitrate in `--channel` (for example `can0@1000000` is invalid).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.

## Damiao Set-Zero Sequence Note

- Recommended order: `disable -> set_zero_position -> enable -> ensure_mode -> control`.
- `set_zero_position` is guarded in core and requires disabled state.


Build from repo root:

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
```

Files:

- `cpp_wrapper_demo.cpp`: Damiao MIT loop
- `robstride_wrapper_demo.cpp`: RobStride ping / read-param / mit / vel demo
- `full_modes_demo.cpp`: Damiao full-mode control
- `pid_register_tune_demo.cpp`: Damiao tuning
- `scan_ids_demo.cpp`: Damiao scan (legacy helper)
- `pos_ctrl_demo.cpp`: Damiao position target
- `pos_repl_demo.cpp`: Damiao interactive position console

Unified scan via Rust CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```
