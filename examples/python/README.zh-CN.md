# Python ctypes 示例

本目录是通过 Python `ctypes` 调用 Rust `motor_abi` 的示例。

> English version: [README.md](README.md)

## 文件

- `python_ctypes_demo.py`：统一多模式 Python 示例（`enable/disable/mit/pos-vel/vel/force-pos`）

## 前置步骤

在项目根目录（`rust_dm`）执行：

```bash
cargo build -p motor_abi --release
```

建议始终在项目根目录运行 Python 脚本，确保相对路径 `.so` 正常加载：

```bash
python3 examples/python/python_ctypes_demo.py --help
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
- `--ensure-mode`：`1/0`，在非使能/失能模式下先确保控制模式
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

## 常见问题

- `OSError: cannot open shared object file`：
  - 先构建：`cargo build -p motor_abi --release`
  - 从项目根目录运行脚本
- `socketcan write failed: Network is down`：
  - 先确认 `can0` 已启用（`ip link show can0`）
- 持续 `no feedback yet`：
  - 检查 `feedback-id`、接线与供电
  - 用 `candump can0` 确认总线有反馈帧
