# motorbridge 文档（中文）

本文档与当前 `main` 分支实现保持同步。

## 快速入口

- 架构说明：[architecture.md](architecture.md)
- CLI 使用：[cli.md](cli.md)
- ABI 接口：[abi.md](abi.md)
- 多语言示例：[examples.md](examples.md)
- 支持设备：[devices.md](devices.md)
- 扩展开发：[extending.md](extending.md)
- 标定工具：[`tools/motor_calib/README.zh-CN.md`](../../tools/motor_calib/README.zh-CN.md)

## motorbridge 提供什么

- 与厂商无关的通用核心（`motor_core`）
- 厂商协议插件（`motor_vendors/*`）
- Rust CLI（`motor_cli`）
- 稳定 C ABI（`motor_abi`，供 C/C++/Python 等调用）
- Python SDK 包（`bindings/python`）

## 建议阅读顺序

1. [architecture.md](architecture.md)
2. [devices.md](devices.md)
3. [cli.md](cli.md)
4. [abi.md](abi.md)
5. [examples.md](examples.md)
6. [extending.md](extending.md)
