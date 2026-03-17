# Python 示例

本目录用于通过 Python `ctypes` 直接调用 Rust `motor_abi`。

> English version: [README.md](README.md)

## 本目录用途

- 演示 Python 侧直接 ABI 调用（`ctypes`）
- 提供完整控制模式命令示例（`enable/disable/mit/pos-vel/vel/force-pos`）
- 作为排查 ABI 问题的最小基线（不依赖 SDK 打包层）

如果你更偏向高层封装，建议使用 `bindings/python` 下的 `motorbridge-cli`。

## 文件

- `python_ctypes_demo.py`：统一多模式 ctypes 示例（`enable/disable/mit/pos-vel/vel/force-pos`）

## 前置步骤

在项目根目录（`rust_dm`）执行：

```bash
cargo build -p motor_abi --release
```

建议从项目根目录运行脚本，确保相对路径 `.so` 加载正常：

```bash
python3 examples/python/python_ctypes_demo.py --help
```

运行前请先配置 CAN：

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

## 通用参数

- `--channel`：CAN 通道（默认 `can0`）
- `--model`：电机型号（默认 `4340`）
- `--motor-id`：命令 ID，例如 `0x01`
- `--feedback-id`：反馈 ID，例如 `0x11`
- `--mode`：`enable|disable|mit|pos-vel|vel|force-pos`
- `--loop`：循环发送次数
- `--dt-ms`：每次循环间隔（毫秒）
- `--print-state`：`1/0`，是否打印反馈状态
- `--ensure-mode`：`1/0`，非 enable/disable 模式下先确保控制模式
- `--ensure-timeout-ms`：确保模式超时（毫秒）

控制参数：

- MIT：`--pos --vel --kp --kd --tau`
- POS_VEL：`--pos --vlim`
- VEL：`--vel`
- FORCE_POS：`--pos --vlim --ratio`

## 完整命令

单独使能：

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100 --print-state 1
```

单独失能：

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100 --print-state 1
```

MIT 模式：

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0.0 --vel 0.0 --kp 20 --kd 1 --tau 0.0 \
  --ensure-mode 1 --ensure-timeout-ms 1000 \
  --loop 200 --dt-ms 20 --print-state 1
```

POS_VEL 模式（到目标位置）：

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 \
  --ensure-mode 1 --ensure-timeout-ms 1000 \
  --loop 300 --dt-ms 20 --print-state 1
```

VEL 模式：

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 \
  --ensure-mode 1 --ensure-timeout-ms 1000 \
  --loop 100 --dt-ms 20 --print-state 1
```

FORCE_POS 模式：

```bash
python3 examples/python/python_ctypes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 \
  --ensure-mode 1 --ensure-timeout-ms 1000 \
  --loop 100 --dt-ms 20 --print-state 1
```

## 相关 SDK CLI（ID/扫描工具）

`id-dump` / `id-set` / `scan` 建议使用 `bindings/python` 的 `motorbridge-cli`：

```bash
motorbridge-cli id-dump --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11
motorbridge-cli id-set --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --new-motor-id 0x02 --new-feedback-id 0x12 --store 1 --verify 1
motorbridge-cli scan --channel can0 --model 4340P --start-id 0x01 --end-id 0x10
```

## 常见问题

- `OSError: cannot open shared object file`：
  - 先构建 ABI：`cargo build -p motor_abi --release`
  - 从项目根目录运行脚本
- `socketcan write failed: Network is down`：
  - 先确认 `can0` 已启用（`ip link show can0`）
- 持续 `no feedback yet`：
  - 检查 model、ID、接线和供电
