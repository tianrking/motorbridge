# Python ctypes Examples

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan + Damiao Serial Bridge)

- Linux SocketCAN uses interface names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- Damiao-only serial bridge transport is also available in CLI (`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`).
- Full Damiao serial-bridge interface list and command patterns are documented in `motor_cli/README.md` (section `3.6` in `motor_cli/README.zh-CN.md`).
- On Linux SocketCAN, do not append bitrate in `--channel` (for example `can0@1000000` is invalid).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.


Python demos that call the Rust ABI directly through `ctypes`.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## File

- `python_ctypes_demo.py`: unified two-vendor demo

Vendor coverage:

- Damiao: `enable`, `disable`, `mit`, `pos-vel`, `vel`, `force-pos`
- RobStride: `ping`, `enable`, `disable`, `mit`, `vel`, `read-param`, `write-param`

## Build and Run

```bash
cargo build -p motor_abi --release
python3 examples/python/python_ctypes_demo.py --help
```

## Examples

Damiao MIT:

```bash
python3 examples/python/python_ctypes_demo.py \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride ping:

```bash
python3 examples/python/python_ctypes_demo.py \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride read parameter:

```bash
python3 examples/python/python_ctypes_demo.py \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019 --param-type f32
```

RobStride write parameter:

```bash
python3 examples/python/python_ctypes_demo.py \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode write-param --param-id 0x700A --param-type f32 --param-value 0.2
```
