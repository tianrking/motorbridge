# 架构说明

## 分层视图

```mermaid
flowchart TB
  APP["上层应用（Rust/C/C++/Python/ROS2/WS）"] --> ABI["motor_abi / SDK 封装层"]
  ABI --> CORE["motor_core（controller/bus/model/traits）"]
  CORE --> VENDOR["motor_vendors/*（协议/寄存器/型号映射）"]
  VENDOR --> CAN["SocketCAN 总线"]
  CAN --> HW["真实电机硬件"]
```

## 运行时控制流

```mermaid
sequenceDiagram
  participant U as 用户工具
  participant C as CoreController
  participant V as Vendor Motor
  participant B as SocketCAN
  U->>C: 发送控制指令（MIT/POS_VEL/VEL/FORCE_POS）
  C->>V: 按电机句柄路由
  V->>B: 编码并发送帧
  B-->>V: 反馈帧
  V-->>C: 解析后的状态
  C-->>U: 状态缓存/回读结果
```

## 设计目标

`motorbridge` 的核心目标是把通用控制能力与厂商协议细节解耦。

- 核心层可跨品牌复用
- 厂商差异下沉到 vendor crate
- 通过 ABI 暴露稳定跨语言接口

## 仓库结构

```text
motorbridge/
├── motor_core/                  # 与厂商无关的运行时
├── motor_vendors/
│   ├── damiao/                  # 生产可用实现
│   └── template/                # 新厂商模板
├── motor_cli/                   # Rust CLI
├── motor_abi/                   # C ABI (cdylib + staticlib)
├── bindings/
│   ├── python/                 # Python SDK 包 + CLI
│   └── cpp/                    # C++ RAII 封装 + CMake 包
├── examples/                    # C/C++/Python 示例
└── docs/
    ├── en/
    └── zh/
```

## 分层说明

### 1) `motor_core`

- `bus.rs`：CAN 总线抽象
- `device.rs`：统一 `MotorDevice` 接口
- `controller.rs`：调度/路由/轮询
- `model.rs`：型号目录抽象
- `socketcan.rs`：Linux SocketCAN 后端

### 2) `motor_vendors/*`

每个厂商 crate 负责：

- 帧编码/解码
- 寄存器语义
- 型号限制参数
- 基于 `CoreController` 的控制器封装

### 3) `motor_abi`

- 导出 C 兼容句柄与函数
- Rust 错误转换为返回码 + `motor_last_error_message()`
- 供 C/C++/Python 等语言集成

### 4) SDK 与示例

- `motor_cli`：运维/调试命令行
- `bindings/python`：可直接集成的 Python 包
- `bindings/cpp`：可直接集成的 C++ RAII 封装
- `examples/*`：最小化跨语言 ABI 调用示例

## 生命周期策略

控制器生命周期显式管理：

- 需要明确停止/失能流程时：调用 `motor_controller_shutdown`
- 只想关闭本地会话/总线时：调用 `motor_controller_close_bus`
- `motor_controller_free` 仅负责释放对象
