# Python 实用示例

与 `bindings/cpp/examples` 对齐的 Python 实用示例：

- `python_wrapper_demo.py`：MIT 循环基础示例
- `full_modes_demo.py`：全模式全参数示例（`enable/disable/mit/pos-vel/vel/force-pos`）
- `pid_register_tune_demo.py`：PID/高影响寄存器调参 + 回读校验
- `scan_ids_demo.py`：总线快速扫描
- `pos_ctrl_demo.py`：单次目标位置控制（POS_VEL）
- `pos_repl_demo.py`：交互式位置控制台

> English version: [README.md](README.md)

## 先配置 CAN

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

## 快速运行

在仓库根目录执行：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/python_wrapper_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

## 全模式命令（全参数）

单独使能：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100 --print-state 1
```

单独失能：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100 --print-state 1
```

MIT（`pos/vel/kp/kd/tau`）：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 200 --dt-ms 20 --print-state 1
```

POS_VEL（`pos/vlim`）：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 300 --dt-ms 20 --print-state 1
```

VEL（`vel`）：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 100 --dt-ms 20 --print-state 1
```

FORCE_POS（`pos/vlim/ratio`）：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 100 --dt-ms 20 --print-state 1
```

## PID 与高影响寄存器调参

常用高影响寄存器：

- PID：`KP_ASR(25)`、`KI_ASR(26)`、`KP_APR(27)`、`KI_APR(28)`
- 映射/动态：`PMAX(21)`、`VMAX(22)`、`TMAX(23)`、`ACC(4)`、`DEC(5)`、`MAX_SPD(6)`

示例：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/pid_register_tune_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --kp-asr 20 --ki-asr 0.2 --kp-apr 30 --ki-apr 0.1 \
  --pmax 12.5 --vmax 45 --tmax 18 --acc 30 --dec -30 --max-spd 30 \
  --store 1 --timeout-ms 1000
```

## 扫描和位置辅助示例

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/scan_ids_demo.py \
  --channel can0 --model 4310 --start-id 0x01 --end-id 0xFF --feedback-base 0x10 --timeout-ms 120
```

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/pos_ctrl_demo.py \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --target-pos 3.14 --vlim 1.5 --loop 300 --dt-ms 20
```

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/pos_repl_demo.py \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 --vlim 1.5
```
