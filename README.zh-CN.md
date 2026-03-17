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
├── integrations/
│   ├── ros2_bridge/         # ROS2 桥接（已实现）
│   └── ws_gateway/          # WebSocket 网关（已实现，V1）
├── tools/
│   └── motor_calib/         # 标定工具（scan / set-id / verify）
├── bindings/
│   ├── python/              # Python SDK 包（pip / motorbridge-cli）
│   └── cpp/                 # C++ RAII 封装 + CMake 包
├── docs/
│   ├── en/                    # 英文文档
│   └── zh/                    # 中文文档
└── examples/
    └── README.md            # 多语言示例索引
```

`motor_vendors/` 是统一的厂商命名空间目录。  
每个子目录对应一个厂商实现（如 `damiao`）或一个接入模板（`template`）。

## 当前支持

详见 [docs/zh/devices.md](docs/zh/devices.md)。

当前正式支持：

- 品牌：**Damiao**
- 型号：`3507`, `4310`, `4310P`, `4340`, `4340P`, `6006`, `8006`, `8009`, `10010L`, `10010`, `H3510`, `G6215`, `H6220`, `JH11`, `6248P`
- 控制模式：`MIT`, `POS_VEL`, `VEL`, `FORCE_POS`

## 能力总览矩阵

### 控制能力

| 能力 | `motor_cli` | `motor_calib` | Python SDK CLI | Python API | C ABI |
|---|---|---|---|---|---|
| 使能 / 失能 | 支持 | 不支持 | 支持（`run --mode enable/disable`） | 支持 | 支持 |
| MIT 指令 | 支持 | 不支持 | 支持 | 支持 | 支持 |
| POS_VEL 指令 | 支持 | 不支持 | 支持 | 支持 | 支持 |
| VEL 指令 | 支持 | 不支持 | 支持 | 支持 | 支持 |
| FORCE_POS 指令 | 支持 | 不支持 | 支持 | 支持 | 支持 |
| 模式确保 | 支持 | 不支持 | 支持 | 支持 | 支持 |
| 电机状态读取 | 支持 | 不支持 | 支持 | 支持 | 支持 |
| 型号握手校验（`PMAX/VMAX/TMAX`） | 支持 | 不支持 | 不支持 | 可手动实现 | 可手动实现 |

### 配置/标定能力

| 能力 | `motor_cli` | `motor_calib` | Python SDK CLI | Python API | C ABI |
|---|---|---|---|---|---|
| 在线 ID 扫描 | 脚本循环方式 | 支持（`scan`） | 支持（`scan`） | 支持（自定义循环） | 支持（自定义循环） |
| 修改 `ESC_ID` / `MST_ID` | 支持（`--set-*`） | 支持（`set-id`） | 支持（`id-set`） | 支持 | 支持 |
| 回读校验 ID | 支持（`--verify-id`） | 支持（`verify`） | 支持（`id-dump`） | 支持 | 支持 |
| 读取关键寄存器（`7/8/9/10/21/22/23`） | 暂无专用命令 | `verify` 可读核心项 | 支持（`id-dump`） | 支持 | 支持 |
| 写寄存器（`u32`/`f32`） | 内置有限能力 | 不支持 | 通过 API | 支持 | 支持 |
| 参数存储（`0xAA`） | 支持（改ID流程） | 支持 | 支持 | 支持 | 支持 |

### 项目优势

- 一套协议核心，多种控制入口（Rust CLI/工具 + Python + C ABI）。
- 既能快速运维（`motor_calib`），也能做工程集成（SDK/ABI）。
- 同一套硬件能力可被多语言、不同运行时一致调用。

## `motor_cli` 与 `motor_calib` 的定位

两者不是重复关系，而是“控制面”和“运维/标定面”的分工：

- `motor_cli`：在线控制主入口，负责使能/失能、MIT、位置速度控制、模式切换与握手校验。
- `motor_calib`：标定运维入口，负责扫描在线 ID、改 ID、回读校验，适合上线前调试与批量维护。

推荐使用方式：

- 日常控制与联调：优先 `motor_cli`。
- 设备编址、换机、排障：优先 `motor_calib`。
- 需要“一把梭”时：可以仅用 `motor_cli`（已支持改 ID），但批量流程仍建议 `motor_calib`。

### `motor_cli` 常用功能与示例

查看帮助：

```bash
cargo run -p motor_cli -- --help
```

1. 单独使能

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100
```

2. 单独失能

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100
```

3. MIT 控制

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 200 --dt-ms 20
```

4. POS_VEL 控制（到目标位置）

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 300 --dt-ms 20
```

5. VEL 控制

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 --loop 100 --dt-ms 20
```

6. FORCE_POS 控制

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 --loop 100 --dt-ms 20
```

7. 改 ID（纯 Rust，`motor_cli` 直接支持）

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4310 --motor-id 0x07 --feedback-id 0x17 \
  --set-motor-id 0x02 --set-feedback-id 0x12 --store 1 --verify-id 1
```

### `motor_calib` 常用功能与示例

查看帮助：

```bash
cargo run -p motor_calib -- --help
```

1. 扫描在线 ID

```bash
cargo run -p motor_calib -- scan \
  --channel can0 --model 4310 --start-id 0x01 --end-id 0x10 --timeout-ms 100
```

2. 改 ID（标准标定流程）

```bash
cargo run -p motor_calib -- set-id \
  --channel can0 --model 4310 \
  --motor-id 0x02 --feedback-id 0x12 \
  --new-motor-id 0x05 --new-feedback-id 0x15 \
  --store 1 --verify 1
```

3. 回读校验 ID

```bash
cargo run -p motor_calib -- verify \
  --channel can0 --model 4310 --motor-id 0x05 --feedback-id 0x15
```

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
- 每次 push / PR 会上传 ABI 多平台产物：
  `linux-x86_64`、`linux-aarch64`、`windows-x86_64`
- 在版本标签（`v*.*.*`）触发时，还会自动发布 Python wheels 到 GitHub Releases：
  Linux（`x86_64` / `aarch64`）、Windows（`x86_64`）
- ABI 发布包同时包含 C++ 封装头文件和 CMake 包配置
  （`find_package(motorbridge CONFIG REQUIRED)`）。
- Linux `x86_64` 额外发布 `.deb` 安装包（`motorbridge-abi`）。
- 用户可以直接从 GitHub Actions 或 GitHub Releases 下载对应产物并按示例调用

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

- 文档总览（英文）：[docs/en/index.md](docs/en/index.md)
- 文档总览（中文）：[docs/zh/index.md](docs/zh/index.md)
- ABI 文档（中文）：[docs/zh/abi.md](docs/zh/abi.md)
- CLI 文档（中文）：[docs/zh/cli.md](docs/zh/cli.md)
- 集成入口：[integrations/README.md](integrations/README.md)
- ROS2 桥接（中文）：[integrations/ros2_bridge/README.zh-CN.md](integrations/ros2_bridge/README.zh-CN.md)
- WS 网关（中文）：[integrations/ws_gateway/README.zh-CN.md](integrations/ws_gateway/README.zh-CN.md)
- 标定工具（中文）：[tools/motor_calib/README.zh-CN.md](tools/motor_calib/README.zh-CN.md)
- C 示例：[examples/c/c_abi_demo.c](examples/c/c_abi_demo.c)
- C++ 示例：[examples/cpp/cpp_abi_demo.cpp](examples/cpp/cpp_abi_demo.cpp)
- Python ctypes 示例：[examples/python/python_ctypes_demo.py](examples/python/python_ctypes_demo.py)
- Python SDK 包：[bindings/python](bindings/python)
- C++ 封装包：[bindings/cpp](bindings/cpp)
- C++ 封装说明（英文）：[bindings/cpp/README.md](bindings/cpp/README.md)
- C++ 封装说明（中文）：[bindings/cpp/README.zh-CN.md](bindings/cpp/README.zh-CN.md)
- Python SDK CLI 子命令：`run` / `id-dump` / `id-set` / `scan`
- 示例总览（英文）：[examples/README.md](examples/README.md)
- 示例总览（中文）：[examples/README.zh-CN.md](examples/README.zh-CN.md)

## 新增品牌

可直接从 [motor_vendors/template](motor_vendors/template) 复制改造。

详细流程见 [docs/zh/extending.md](docs/zh/extending.md)。

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
