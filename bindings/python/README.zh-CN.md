# motorbridge Python SDK

用于调用 `motorbridge` Rust ABI 的 Python 包。

> English: [README.md](README.md)

## 安装

### A) 从 Release wheel 安装（推荐给使用者）

```bash
pip install motorbridge-0.1.0-<python_tag>-linux_x86_64.whl
```

示例：

```bash
pip install motorbridge-0.1.0-cp310-cp310-linux_x86_64.whl
```

### B) 本地可编辑安装（推荐给开发者）

在仓库根目录执行：

```bash
cd bindings/python
pip install -e .
```

运行前先构建一次 Rust ABI（本地开发路径）：

```bash
cd ../../
cargo build -p motor_abi --release
```

## Python API 快速使用

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    m = ctrl.add_damiao_motor(0x01, 0x11, "4340P")
    ctrl.enable_all()
    m.ensure_mode(Mode.MIT, 1000)
    m.send_mit(0.0, 0.0, 20.0, 1.0, 0.0)
    print(m.get_state())
    m.close()
```

## 命令行

```bash
motorbridge-cli --help
```

## CLI 参数说明

通用参数：

- `--channel`（默认 `can0`）
- `--model`（默认 `4340`）
- `--motor-id`（默认 `0x01`）
- `--feedback-id`（默认 `0x11`）
- `--mode`（`enable|disable|mit|pos-vel|vel|force-pos`）
- `--loop`（默认 `100`）
- `--dt-ms`（默认 `20`）
- `--print-state`（`1/0`，默认 `1`）
- `--ensure-mode`（`1/0`，默认 `1`，仅非 enable/disable）
- `--ensure-timeout-ms`（默认 `1000`）
- `--ensure-strict`（`1/0`，默认 `0`）

模式参数：

- MIT：`--pos --vel --kp --kd --tau`
- POS_VEL：`--pos --vlim`
- VEL：`--vel`
- FORCE_POS：`--pos --vlim --ratio`

## 完整 CLI 命令

使能：

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100 --print-state 1
```

失能：

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100 --print-state 1
```

MIT：

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 200 --dt-ms 20 --print-state 1
```

POS_VEL：

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 300 --dt-ms 20 --print-state 1
```

VEL：

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 100 --dt-ms 20 --print-state 1
```

FORCE_POS：

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 100 --dt-ms 20 --print-state 1
```

## 动态库加载顺序

1. 环境变量 `MOTORBRIDGE_LIB`
2. 包内 `motorbridge/lib/*`
3. 仓库 `target/release/*`
4. `ctypes.util.find_library("motor_abi")`

## 常见问题

- `Failed to load motor_abi shared library`：
  - 确认 wheel 包含 `motorbridge/lib/libmotor_abi.so`
  - 或设置 `MOTORBRIDGE_LIB=/path/to/libmotor_abi.so`
- `socketcan write failed: Network is down`：
  - 先确认并启用 `can0`（`ip link show can0`）
- 持续 `no feedback yet`：
  - 检查 `feedback-id`、总线接线和供电
