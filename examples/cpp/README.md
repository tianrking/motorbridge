# C++ ABI Examples

Direct C++ examples on top of `motor_abi.h`.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Files

- `cpp_abi_demo.cpp`: unified ABI demo for Damiao and RobStride

## Build

```bash
cargo build -p motor_abi --release
g++ -std=c++17 examples/cpp/cpp_abi_demo.cpp -I motor_abi/include -L target/release -lmotor_abi -o cpp_abi_demo
LD_LIBRARY_PATH=target/release ./cpp_abi_demo --help
```

## Examples

Damiao MIT:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride ping:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride read parameter:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019 --param-type f32
```
