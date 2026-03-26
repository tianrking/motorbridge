# Python 实用示例（Damiao 完整用法）

本文件只聚焦 Damiao：把 Python 绑定里 Damiao 常用接口、四种控制模式、参数配置和完整执行顺序讲清楚。

> English version: [README.md](README.md)

## 1) 通道与运行前提

1. Linux SocketCAN 直接用 `can0/can1/slcan0`。  
2. Linux 下 `--channel` 不要写 `@bitrate`（例如 `can0@1000000` 无效）。  
3. Windows PCAN 后端中，`can0/can1` 映射 `PCAN_USBBUS1/2`（可带 `@bitrate`）。  
4. 运行示例前建议设置：

```bash
export PYTHONPATH=bindings/python/src
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}
```

## 2) Damiao 标准执行流程（必须理解）

1. 创建控制器并绑定电机（`Controller` + `add_damiao_motor`）。  
2. 上电使能（`ctrl.enable_all()`）。  
3. 切换或确认控制模式（`motor.ensure_mode(...)`）。  
4. 按模式发送命令（`send_mit/send_pos_vel/send_vel/send_force_pos`）。  
5. 按需读反馈（`request_feedback/get_state` 或脚本中的打印）。  
6. 测试结束失能（`motor.disable()` 或 `ctrl.disable_all()`）。

## 3) Damiao 接口总览（Python 绑定）

### 3.1 Controller 侧

- `Controller(channel="can0")`
- `Controller.from_dm_serial(serial_port="/dev/ttyACM0", baud=921600)`
- `add_damiao_motor(motor_id, feedback_id, model)`
- `enable_all()`
- `disable_all()`
- `poll_feedback_once()`
- `shutdown()`
- `close_bus()`

### 3.2 Motor 侧

- 基础控制：
  - `enable()`
  - `disable()`
  - `ensure_mode(mode, timeout_ms)`
  - `send_mit(pos, vel, kp, kd, tau)`
  - `send_pos_vel(pos, vlim)`
  - `send_vel(vel)`
  - `send_force_pos(pos, vlim, ratio)`
- 反馈与维护：
  - `request_feedback()`
  - `get_state()`
  - `clear_error()`
  - `set_zero_position()`
  - `set_can_timeout_ms(timeout_ms)`
  - `store_parameters()`
- 寄存器读写：
  - `get_register_f32(rid, timeout_ms)`
  - `write_register_f32(rid, value)`
  - `get_register_u32(rid, timeout_ms)`
  - `write_register_u32(rid, value)`

## 4) 四种模式怎么用（方法 + 参数）

### 4.1 MIT（全参数混合控制）

方法：`send_mit(pos, vel, kp, kd, tau)`  
参数：

- `pos`：目标位置，单位 `rad`
- `vel`：目标速度，单位 `rad/s`
- `kp`：位置刚度（无量纲）
- `kd`：速度阻尼（无量纲）
- `tau`：前馈力矩，单位 `Nm`

常用起步参数：`kp=2~10`, `kd=0.05~0.5`, `tau=0.1~1.0`

### 4.2 POS_VEL（位置 + 速度上限）

方法：`send_pos_vel(pos, vlim)`  
参数：

- `pos`：目标位置，单位 `rad`
- `vlim`：速度上限，单位 `rad/s`

常用场景：先安全到位，再切其它模式。

### 4.3 VEL（纯速度）

方法：`send_vel(vel)`  
参数：

- `vel`：目标速度，单位 `rad/s`

常用场景：连续旋转、速度响应测试。

### 4.4 FORCE_POS（位置 + 力矩限幅）

方法：`send_force_pos(pos, vlim, ratio)`  
参数：

- `pos`：目标位置，单位 `rad`
- `vlim`：速度上限，单位 `rad/s`
- `ratio`：力矩/电流限幅比例，范围 `0~1`（不是精确恒扭矩）

常用场景：需要到位，但不想“死顶”太狠。

## 5) Damiao 示例脚本与用途

- `python_wrapper_demo.py`：MIT 最小闭环
- `full_modes_demo.py`：四模式统一入口
- `scan_ids_demo.py`：Damiao 扫描
- `pos_ctrl_demo.py`：单次位置目标
- `multi_motor_ctrl_demo.py`：多电机控制（`-id/-pos` 一一对应）+ SDK 每步耗时日志
- `mit_pos_switch_demo.py`：多电机模式切换测试（MIT -> POS_VEL）
- `pos_repl_demo.py`：交互式位置控制
- `pid_register_tune_demo.py`：寄存器调参
- `damiao_maintenance_demo.py`：维护接口（清错、置零、超时、反馈）
- `damiao_register_rw_demo.py`：寄存器 f32/u32 读写 + 存参
- `damiao_dm_serial_demo.py`：Damiao 串口桥传输链路
- `dm_serial_mode_switch_200_demo.py`：`dm-serial` 连续切换四模式（每模式可设循环次数，默认 200）
- `dm_serial_status_like_cli_demo.py`：`dm-serial` 查询当前模式 + 关键寄存器 + 实时状态（类 CLI 输出）
- `dm_serial_leader_monitor_demo.py`：`dm-serial` leader 监控（指定 ID 全状态上报 + 启动时全体使能）

## 6) 按顺序实操命令（推荐）

### 6.1 扫描

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/scan_ids_demo.py \
  --channel can0 --model 4310 --start-id 1 --end-id 16
```

### 6.2 MIT（最小验证）

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/python_wrapper_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0.3 --loop 100 --dt-ms 10
```

### 6.3 四模式统一入口

```bash
# POS_VEL
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos -2.0 --vlim 1.0 --loop 100 --dt-ms 20

# VEL
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 1.0 --loop 100 --dt-ms 20

# FORCE_POS
PYTHONPATH=bindings/python/src python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos -2.0 --vlim 1.0 --ratio 0.2 --loop 100 --dt-ms 10
```

### 6.4 维护接口

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/damiao_maintenance_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --can-timeout-ms 1000 --set-zero 0
```

### 6.5 寄存器读写

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/damiao_register_rw_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --read-f32-rid 21 --read-u32-rid 10 --store 0
```

### 6.6 串口桥链路（Damiao 专用）

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/damiao_dm_serial_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x01 --feedback-id 0x11 --mode mit --loop 40 --dt-ms 20
```

### 6.7 多电机位置控制 + 每步 SDK 计时（4号与7号示例）

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/multi_motor_ctrl_demo.py \
  --channel can0 --model 4310 --mode pos-vel \
  -id 4 7 -pos 0.8 -0.6 -vlim 1.2 1.2 \
  --loop 50 --dt-ms 20 --trace-sdk 1 --timing-log 1 --timing-every 5
```

说明：

- `-id` 后可传多个电机 ID（空格或逗号都可）。
- `-pos` 与 `-id` 顺序一一对应。
- `--trace-sdk 1`：打印每次 SDK 调用耗时（`add/enable/ensure/send/get_state/close`）。
- `--timing-log 1`：打印汇总统计（平均、最小、最大、总耗时）。

### 6.8 模式切换测试（MIT 与 POS_VEL，4号与7号示例）

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/mit_pos_switch_demo.py \
  --channel can0 --model 4310 -id 4 7 \
  --trajectory -3 \
  --mit-hold-loops 0 --pos-hold-loops 50 \
  --dt-ms 20 --print-state 1
```

说明：

- `--mit-hold-loops 0` 表示仅验证 MIT 切换，不发 MIT 控制循环。
- `--pos-hold-loops 50` 表示在 POS_VEL 下执行 50 个循环。

### 6.9 `dm-serial` 专用：四模式切换证明 + 状态查询

```bash
# A) 四模式切换（MIT -> POS_VEL -> VEL -> FORCE_POS；每模式 200 次）
PYTHONPATH=bindings/python/src LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH} \
python3 bindings/python/examples/dm_serial_mode_switch_200_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x07 --feedback-id 0x17 --ensure-timeout-ms 300 \
  --loop-per-mode 200 --dt-ms 20

# B) 状态查询（当前模式 + CTRL_MODE/ID/TIMEOUT/PMAX/VMAX/TMAX + pos/vel/torq/status）
PYTHONPATH=bindings/python/src LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH} \
python3 bindings/python/examples/dm_serial_status_like_cli_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x07 --feedback-id 0x17 --timeout-ms 300 --loop 50 --dt-ms 100
```

## 7) 结束动作（务必执行）

推荐在结束前明确失能：

```python
motor.disable()
# 或
ctrl.disable_all()
```

这样可以避免电机在使能状态下持续输出。
