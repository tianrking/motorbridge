# 示例索引

这里是当前 `motorbridge` 跨语言示例的总入口。

> English version: [README.md](README.md)

## 覆盖范围

- Rust CLI: `motor_cli/src/main.rs`
- C ABI 示例: `examples/c/c_abi_demo.c`
- C++ ABI 示例: `examples/cpp/cpp_abi_demo.cpp`
- Python ctypes 示例: `examples/python/python_ctypes_demo.py`
- Python SDK 示例: `bindings/python/examples/*`
- C++ wrapper 示例: `bindings/cpp/examples/*`
- Damiao 调参总表:
  - `examples/dm_api.md`
  - `examples/dm_api_cn.md`

## 示例支持的厂商

- Damiao:
  - 模式: `enable`、`disable`、`mit`、`pos-vel`、`vel`、`force-pos`
  - 寄存器和改 ID 流程仍主要走 CLI、Python SDK 和校准工具
- RobStride:
  - 模式: `ping`、`enable`、`disable`、`mit`、`vel`、`read-param`、`write-param`
  - 参数示例走 RobStride 的 ABI / binding 接口

## CAN 初始化

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

## 快速开始

Damiao 的 Rust CLI 示例:

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride 的 Rust CLI 示例:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode ping
```

RobStride 读参数:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019
```

## 跨语言 ABI 示例

Python ctypes:

```bash
cargo build -p motor_abi --release
python3 examples/python/python_ctypes_demo.py --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
python3 examples/python/python_ctypes_demo.py --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

C:

```bash
cargo build -p motor_abi --release
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
LD_LIBRARY_PATH=target/release ./c_abi_demo --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
LD_LIBRARY_PATH=target/release ./c_abi_demo --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

C++:

```bash
cargo build -p motor_abi --release
g++ -std=c++17 examples/cpp/cpp_abi_demo.cpp -I motor_abi/include -L target/release -lmotor_abi -o cpp_abi_demo
LD_LIBRARY_PATH=target/release ./cpp_abi_demo --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
LD_LIBRARY_PATH=target/release ./cpp_abi_demo --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

## 推荐的高层示例

- Python SDK:
  - `bindings/python/examples/python_wrapper_demo.py`
  - `bindings/python/examples/robstride_wrapper_demo.py`
- C++ wrapper:
  - `bindings/cpp/examples/cpp_wrapper_demo.cpp`
  - `bindings/cpp/examples/robstride_wrapper_demo.cpp`

## 说明

- `id-dump`、`id-set` 仍偏 Damiao 工作流；统一 `scan` 已支持 Rust CLI（`--vendor all`）和 Python SDK CLI（`motorbridge.cli scan --vendor all`）。
- RobStride 目前重点覆盖 `ping`、参数访问、MIT、速度控制。
