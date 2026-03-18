# C++ ABI 示例

这里是基于 `motor_abi.h` 的 C++ 直接示例。

> English version: [README.md](README.md)

## 文件

- `cpp_abi_demo.cpp`: 同时支持 Damiao 和 RobStride 的统一 ABI 示例

## 构建

```bash
cargo build -p motor_abi --release
g++ -std=c++17 examples/cpp/cpp_abi_demo.cpp -I motor_abi/include -L target/release -lmotor_abi -o cpp_abi_demo
LD_LIBRARY_PATH=target/release ./cpp_abi_demo --help
```

## 示例

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

RobStride 读参数:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019 --param-type f32
```
