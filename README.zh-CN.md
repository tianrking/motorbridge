# motorbridge

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/tianrking/motorbridge/actions/workflows/ci.yml/badge.svg)](https://github.com/tianrking/motorbridge/actions/workflows/ci.yml)

一个面向 CAN 电机的统一、高可靠控制栈。

仓库地址：https://github.com/tianrking/motorbridge.git

> English version: [README.md](README.md)

## 项目目标

`motorbridge` 的核心思想是把 **通用控制能力** 和 **厂商协议细节** 分层解耦。

- 一套核心运行时负责总线、调度、多设备路由
- 厂商插件负责协议/寄存器/型号差异
- 提供稳定 C ABI，方便 C/C++/Python 等语言调用
- 后续新增品牌不需要重写核心层

## 技术栈与构建语言

- 核心开发语言：**Rust**（edition 2021）
- 底层总线后端：**Linux SocketCAN**（系统调用/FFI）
- 跨语言接口：**C ABI**（`cdylib` + `staticlib`）
- 调用方语言：Rust / C / C++ / Python（`ctypes`）

## 架构说明

```text
motorbridge/
├── motor_core/              # 通用核心（与厂商无关）
│   ├── bus.rs               # CAN 总线抽象
│   ├── device.rs            # 统一 MotorDevice 接口
│   ├── controller.rs        # CoreController 调度与路由
│   ├── model.rs             # 型号目录抽象
│   └── socketcan.rs         # Linux SocketCAN 后端
├── motor_vendors/
│   ├── damiao/              # Damiao 插件（协议/寄存器/型号）
│   └── template/            # 新增品牌模板
├── motor_cli/               # 统一 CLI（模式/参数控制）
├── motor_abi/               # C ABI（cdylib/staticlib）
├── bindings/
│   └── python/              # Python SDK 包（pip / motorbridge-cli）
├── docs/
│   ├── SUPPORTED_DEVICES.md
│   ├── ABI_USAGE.md
│   ├── EXTENDING.md
│   └── DAMIAO_PARITY.md
└── examples/
    └── README.md            # 多语言示例索引
```

`motor_vendors/` 是统一的厂商命名空间目录。  
每个子目录对应一个厂商实现（如 `damiao`）或一个接入模板（`template`）。

## 当前支持

详见 [docs/SUPPORTED_DEVICES.md](docs/SUPPORTED_DEVICES.md)。

当前正式支持：

- 品牌：**Damiao**
- 型号：`3507`, `4310`, `4310P`, `4340`, `4340P`, `6006`, `8006`, `8009`, `10010L`, `10010`, `H3510`, `G6215`, `H6220`, `JH11`, `6248P`
- 控制模式：`MIT`, `POS_VEL`, `VEL`, `FORCE_POS`

## 构建

```bash
cargo check
cargo build --release
```

只构建 core + Damiao：

```bash
cargo build -p motor_core -p motor_vendor_damiao --release
# note: crate motor_vendor_damiao is located at motor_vendors/damiao
```

只构建 ABI：

```bash
cargo build -p motor_abi --release
```

ABI 产物：

- `target/release/libmotor_abi.so`
- `target/release/libmotor_abi.a`

GitHub CI 预构建 ABI 产物：

- 工作流：`.github/workflows/build-abi.yml`
- 每次 push / PR 都会上传多平台产物（`linux` / `macos` / `windows`）
- 其他人可直接从 GitHub Actions 下载对应平台动态库，然后按 ABI 示例调用

## 快速开始（CLI）

查看 CLI 参数：

```bash
cargo run -p motor_cli -- --help
```

MIT 示例：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 30 --kd 1 --tau 0 --loop 200 --dt-ms 20
```

POS_VEL 示例：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 1.2 --vlim 2.0 --loop 100 --dt-ms 20
```

POS_VEL 到目标位置示例（让电机到达并保持在目标位置附近）：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 300 --dt-ms 20
```

型号握手校验（默认开启）：

- CLI 启动时会读取 `PMAX/VMAX/TMAX`（`rid=21/22/23`）。
- 若 `--model` 与设备参数不匹配，会直接报错并给出建议型号。
- 仅在你明确要跳过时使用：`--verify-model 0`。

快速测试命令：

```bash
# 1) 预期通过（型号正确）
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 5 --dt-ms 100

# 2) 预期失败（故意写错型号，应提示建议型号）
cargo run -p motor_cli --release -- \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 5 --dt-ms 100
```

## ABI 与跨语言调用

- ABI 说明：[docs/ABI_USAGE.md](docs/ABI_USAGE.md)
- C 示例：[examples/c/c_abi_demo.c](examples/c/c_abi_demo.c)
- C++ 示例：[examples/cpp/cpp_abi_demo.cpp](examples/cpp/cpp_abi_demo.cpp)
- Python ctypes 示例：[examples/python/python_ctypes_demo.py](examples/python/python_ctypes_demo.py)
- Python SDK 包：[bindings/python](bindings/python)
- Python SDK CLI 子命令：`run` / `id-dump` / `id-set` / `scan`
- 示例总览（英文）：[examples/README.md](examples/README.md)
- 示例总览（中文）：[examples/README.zh-CN.md](examples/README.zh-CN.md)

## 新增品牌

可直接从 [motor_vendors/template](motor_vendors/template) 复制改造。

详细流程见 [docs/EXTENDING.md](docs/EXTENDING.md)。

## 说明

- 当前实现的是 Linux SocketCAN 后端。
- 对具体型号/固件版本，仍建议做实机回归验证。
- `motor_cli` 的 `enable/disable` 模式现在在退出时只关闭本地总线会话，不会隐式自动 `disable` 电机。

## 社区与协作

- 贡献指南：[CONTRIBUTING.md](CONTRIBUTING.md)
- 社区行为准则：[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- 安全策略：[SECURITY.md](SECURITY.md)
- 变更日志：[CHANGELOG.md](CHANGELOG.md)

## 许可证

本项目使用 MIT 许可证，见 [LICENSE](LICENSE)。
