# motorbridge

这是一个统一的 CAN 电机控制栈，包含 vendor-agnostic Rust core、稳定 C ABI，以及 Python/C++ bindings。

> English version: [README.md](README.md)

## 当前支持的厂商

- Damiao:
  - 型号: `3507`, `4310`, `4310P`, `4340`, `4340P`, `6006`, `8006`, `8009`, `10010L`, `10010`, `H3510`, `G6215`, `H6220`, `JH11`, `6248P`
  - 模式: `scan`, `MIT`, `POS_VEL`, `VEL`, `FORCE_POS`
- RobStride:
  - 型号: `rs-00`, `rs-01`, `rs-02`, `rs-03`, `rs-04`, `rs-05`, `rs-06`
  - 模式: `scan`, `scan-manual`, `ping`, `MIT`, `VEL`, 参数读写

## 架构

- `motor_core`: 与厂商无关的控制器、路由、SocketCAN 总线层
- `motor_vendors/damiao`: Damiao 协议 / 型号 / 寄存器
- `motor_vendors/robstride`: RobStride 扩展 CAN 协议 / 型号 / 参数
- `motor_cli`: 统一 Rust CLI
- `motor_abi`: 稳定 C ABI
- `bindings/python`: Python SDK + `motorbridge-cli`
- `bindings/cpp`: C++ RAII wrapper

## 快速开始

构建:

```bash
cargo build
```

拉起 CAN:

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

Damiao CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

RobStride CLI 读参数:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019
```

统一全品牌扫描:

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```

## ABI 与绑定

- C ABI:
  - Damiao: `motor_controller_add_damiao_motor(...)`
  - RobStride: `motor_controller_add_robstride_motor(...)`
- Python:
  - `Controller.add_damiao_motor(...)`
  - `Controller.add_robstride_motor(...)`
- C++:
  - `Controller::add_damiao_motor(...)`
  - `Controller::add_robstride_motor(...)`

RobStride 专属 ABI / binding 能力包括:

- `robstride_ping`
- `robstride_get_param_*`
- `robstride_write_param_*`

## 示例入口

- 跨语言索引: `examples/README.md`
- C ABI 示例: `examples/c/c_abi_demo.c`
- C++ ABI 示例: `examples/cpp/cpp_abi_demo.cpp`
- Python ctypes 示例: `examples/python/python_ctypes_demo.py`
- Python SDK 文档: `bindings/python/README.md`
- C++ binding 文档: `bindings/cpp/README.md`

## Release 资产使用指南

- Ubuntu x86_64 上做 C/C++ 开发：
  - 下载 `motorbridge-abi-<tag>-linux-x86_64.deb`
  - 安装：`sudo apt install ./motorbridge-abi-<tag>-linux-x86_64.deb`
- 其他平台的 C/C++ 开发：
  - 使用 ABI 压缩包（`motorbridge-abi-<tag>-linux-*.tar.gz` 或 `windows-*.zip`）
  - 从包内 include/lib 链接 `libmotor_abi`。
- Python 开发：
  - 下载匹配解释器与平台的 wheel（`cp310/cp311/cp312` + 对应 arch）
  - 安装：`pip install ./motorbridge-*.whl`
  - 或安装源码包：`pip install ./motorbridge-*.tar.gz`
- 设备矩阵: `docs/zh/devices.md`
