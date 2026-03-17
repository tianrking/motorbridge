# motorbridge Python SDK

用于调用 `motorbridge` Rust ABI 的 Python 包。

> English version: [README.md](README.md)

## 安装

### A) 安装发行版 wheel（推荐）

```bash
pip install motorbridge-0.1.0-<python_tag>-<platform>.whl
```

示例：

```bash
pip install motorbridge-0.1.0-cp310-cp310-linux_x86_64.whl
```

### B) 本地可编辑安装（开发）

```bash
cd bindings/python
pip install -e .
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

## Damiao 完整调参参考

如果你要查看 Damiao 的完整控制/调参接口（控制模式、寄存器语义、可调范围、调参流程），请看：

- [DAMIAO_API.zh-CN.md](DAMIAO_API.zh-CN.md)（中文）
- [DAMIAO_API.md](DAMIAO_API.md)（英文）
- [../examples/dm_api_cn.md](../examples/dm_api_cn.md)（完整参数表 + CLI 示例）
- [../examples/dm_api.md](../examples/dm_api.md)（full table + CLI examples）

## CLI 总览

`motorbridge-cli` 现在支持子命令：

- `run`：控制循环（默认命令；兼容旧版平铺参数）
- `id-dump`：读取关键寄存器（默认 `7,8,9,10,21,22,23`）
- `id-set`：设置 `ESC_ID`/`MST_ID`（寄存器 `8`/`7`）并可选回读校验
- `scan`：按 ID 范围探测在线电机

查看帮助：

```bash
motorbridge-cli --help
motorbridge-cli run --help
motorbridge-cli id-dump --help
motorbridge-cli id-set --help
motorbridge-cli scan --help
```

## `run` 命令参数

通用参数：

- `--channel`（默认 `can0`）
- `--model`（默认 `4340`）
- `--motor-id`（默认 `0x01`）
- `--feedback-id`（默认 `0x11`）
- `--mode`（`enable|disable|mit|pos-vel|vel|force-pos`）
- `--loop`（默认 `100`）
- `--dt-ms`（默认 `20`）
- `--print-state`（`1/0`，默认 `1`）
- `--ensure-mode`（`1/0`，默认 `1`，仅对非 enable/disable 生效）
- `--ensure-timeout-ms`（默认 `1000`）
- `--ensure-strict`（`1/0`，默认 `0`）

模式参数：

- MIT：`--pos --vel --kp --kd --tau`
- POS_VEL：`--pos --vlim`
- VEL：`--vel`
- FORCE_POS：`--pos --vlim --ratio`

示例：

```bash
# 单独使能
motorbridge-cli run \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100 --print-state 1

# 单独失能
motorbridge-cli run \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100 --print-state 1

# MIT
motorbridge-cli run \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 200 --dt-ms 20 --print-state 1

# POS_VEL（目标位置）
motorbridge-cli run \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 300 --dt-ms 20 --print-state 1
```

旧命令兼容（仍可用）：

```bash
motorbridge-cli --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

## ID/扫描命令

读取关键寄存器：

```bash
motorbridge-cli id-dump \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --timeout-ms 500
```

修改 ID（写 `ESC_ID`/`MST_ID`）：

```bash
motorbridge-cli id-set \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --new-motor-id 0x02 --new-feedback-id 0x12 \
  --store 1 --verify 1 --timeout-ms 800
```

扫描 ID 范围：

```bash
motorbridge-cli scan \
  --channel can0 --model 4340P \
  --start-id 0x01 --end-id 0x10 --feedback-base 0x10 --timeout-ms 80
```

## 动态库加载顺序

优先级：

1. 环境变量 `MOTORBRIDGE_LIB`
2. 包内 `motorbridge/lib/*`
3. 仓库 `target/release/*`
4. `ctypes.util.find_library("motor_abi")`

## 常见问题

- `Failed to load motor_abi shared library`：
  - 确认 wheel 内含 `motorbridge/lib/libmotor_abi.so`（或对应平台动态库）
  - 或设置 `MOTORBRIDGE_LIB=/path/to/libmotor_abi.so`
- `socketcan write failed: Network is down`：
  - 先确保 CAN 网口已启用（`ip link show can0`）
- 持续 `no feedback yet`：
  - 检查 model、`motor-id`、`feedback-id`、接线和供电
