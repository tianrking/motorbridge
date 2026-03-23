# C ABI Examples

<!-- channel-compat-note -->
## Channel Compatibility (PCAN + slcan)

- Linux uses SocketCAN channel names directly: `can0`, `can1`, `slcan0`.
- For USB-serial CAN adapters, bring up `slcan0` first: `sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`.
- On Linux, do not append bitrate in `--channel` (for example `can0@1000000` is invalid on SocketCAN).
- On Windows (PCAN backend), `can0/can1` map to `PCAN_USBBUS1/2`; optional `@bitrate` suffix is supported.


Direct C demos for `motor_abi`.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Files

- `c_abi_demo.c`: unified demo for both vendors

Vendor coverage:

- Damiao: `enable`, `disable`, `mit`, `pos-vel`, `vel`, `force-pos`
- RobStride: `ping`, `enable`, `disable`, `mit`, `vel`, `read-param`, `write-param`

## Build

```bash
cargo build -p motor_abi --release
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
LD_LIBRARY_PATH=target/release ./c_abi_demo --help
```

## Examples

Damiao MIT:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride ping:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride read position parameter:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019 --param-type f32
```

RobStride low-gain MIT:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0 --loop 20 --dt-ms 50
```
