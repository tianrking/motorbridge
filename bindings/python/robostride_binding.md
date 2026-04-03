# RobStride Python Binding 完整使用文档

> 文件位置：`rust_dm/bindings/python/robostride_binding.md`
> 适用项目：`/home/w0x7ce/Downloads/dm_candrive/rust_dm`

## 0. 现状说明

仓库里**已有** RobStride 相关说明，但分散在多个文件：

- `bindings/python/README.zh-CN.md`（Python 绑定总览，含 RobStride 片段）
- `bindings/python/examples/robstride_wrapper_demo.py`（示例脚本）
- `motor_cli/ROBSTRIDE_API.zh-CN.md`（CLI 维度的 RobStride 协议/参数）

目前 `bindings/python` 目录下没有独立的 RobStride 绑定“单文件完整手册”，所以本文件作为集中版。

## 1. 先编译（Cargo）+ 最小环境验证

在仓库根目录执行：

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
cargo build -p motor_abi --release
```

最小 Python 导入验证：

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
PYTHONPATH=bindings/python/src LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH} python3 - <<'PY'
from motorbridge import Controller, Mode
print('import_ok', Controller.__name__, int(Mode.MIT))
PY
```

若输出 `import_ok Controller 1`，说明 binding 与 ABI 动态库可被正常加载。

## 2. CAN 链路准备（Linux）

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

说明：Linux 下 `--channel` 使用 `can0/slcan0`，不要写 `can0@1000000`。

## 3. Python Binding 的 RobStride 能力总览

### 3.1 控制器与挂载

- `Controller(channel="can0")`
- `Controller.add_robstride_motor(motor_id, feedback_id, model)`

常用默认值建议：

- `model`: `rs-00` 或实际型号（`rs-06` 等）
- `feedback_id`: `0xFF`（常用 host id）

### 3.2 通用控制方法（统一接口）

- `motor.enable()` / `motor.disable()`
- `motor.ensure_mode(Mode.MIT|Mode.POS_VEL|Mode.VEL, timeout_ms)`
- `motor.send_mit(pos, vel, kp, kd, tau)`
- `motor.send_vel(vel)`
- `motor.get_state()`

### 3.3 RobStride 专属方法

- `motor.robstride_ping()`
- `motor.robstride_set_device_id(new_id)`
- `motor.robstride_get_param_{i8,u8,u16,u32,f32}(param_id, timeout_ms)`
- `motor.robstride_write_param_{i8,u8,u16,u32,f32}(param_id, value)`

### 3.3.1 私有协议到顶层统一协议映射表

| 顶层统一模式 | RobStride 原生 | 说明 |
| --- | --- | --- |
| `Mode.MIT` | `run_mode=0(Mit)` | 统一 `send_mit(...)` 直连原生 MIT 控制帧 |
| `Mode.POS_VEL` | `run_mode=1(Position)` + `0x7017(limit_spd)` + `0x7016(loc_ref)` | 统一 `send_pos_vel(pos, vlim)` 会映射到 Position 模式 |
| `Mode.VEL` | `run_mode=2(Velocity)` + `0x700A(spd_ref)` | 统一 `send_vel(vel)` 映射到原生速度目标 |
| `Mode.FORCE_POS` | 不支持 | 当前 ABI/绑定层返回 unsupported |

### 3.4 每个 Python binding 方法对应的原生 CLI

先定义统一前缀（后面所有命令都基于它）：

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
MC='cargo run -p motor_cli --release --'
BASE="--vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF"
```

`Controller(channel=\"can0\") + add_robstride_motor(...)` 对应为设置目标参数前缀：

```bash
$MC $BASE --mode ping
```

`motor.enable()`：

```bash
$MC $BASE --mode enable
```

`motor.disable()`：

```bash
$MC $BASE --mode disable
```

`motor.ensure_mode(Mode.MIT, timeout_ms)`（CLI 里通过控制命令触发/确保）：

```bash
$MC $BASE --mode mit --pos 0 --vel 0 --kp 0.5 --kd 0.2 --tau 0 --loop 1 --dt-ms 20 --ensure-mode 1
```

`motor.ensure_mode(Mode.VEL, timeout_ms)`（CLI 里通过控制命令触发/确保）：

```bash
$MC $BASE --mode vel --vel 0.0 --loop 1 --dt-ms 20 --ensure-mode 1
```

`motor.ensure_mode(Mode.POS_VEL, timeout_ms)`（映射到 RobStride 原生 Position）：

```bash
$MC $BASE --mode pos-vel --pos 1.0 --vlim 1.5 --loop 1 --dt-ms 20 --ensure-mode 1
```

`motor.send_mit(pos, vel, kp, kd, tau)`：

```bash
$MC $BASE --mode mit --pos 0.0 --vel 0.0 --kp 0.5 --kd 0.2 --tau 0.0 --loop 40 --dt-ms 50
```

`motor.send_vel(vel)`：

```bash
$MC $BASE --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

`motor.get_state()`（无单独 mode；CLI 在控制循环中打印状态）：

```bash
$MC $BASE --mode vel --vel 0.0 --loop 1 --dt-ms 20
```

`motor.robstride_ping()`：

```bash
$MC $BASE --mode ping
```

`motor.robstride_set_device_id(new_device_id)`：

```bash
$MC $BASE --set-motor-id 126 --store 1
```

`motor.robstride_get_param_{i8/u8/u16/u32/f32}(param_id, timeout_ms)`：

```bash
$MC $BASE --mode read-param --param-id 0x7019
```

`motor.robstride_write_param_{i8/u8/u16/u32/f32}(param_id, value)`：

```bash
$MC $BASE --mode write-param --param-id 0x700A --param-value 0.3
```

## 4. 可直接运行的最小 Python 示例

```python
from motorbridge import Controller, Mode
import time

CHANNEL = "can0"
MODEL = "rs-06"
MOTOR_ID = 127
FEEDBACK_ID = 0xFF

with Controller(CHANNEL) as ctrl:
    m = ctrl.add_robstride_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    try:
        # 1) 连通性
        device_id, responder_id = m.robstride_ping()
        print("ping:", device_id, responder_id)

        # 2) 读参数（机械位置 0x7019）
        pos = m.robstride_get_param_f32(0x7019, 500)
        print("mechPos:", pos)

        # 3) 速度模式短运行
        ctrl.enable_all()
        m.ensure_mode(Mode.VEL, 1000)
        for _ in range(20):
            m.send_vel(0.3)
            print(m.get_state())
            time.sleep(0.05)

        # 4) 停止
        m.send_vel(0.0)
    finally:
        m.close()
```

运行方式：

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
PYTHONPATH=bindings/python/src LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH} python3 your_script.py
```

CLI 对照（按示例流程最常用 4 条）：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode ping
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode read-param --param-id 0x7019
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode vel --vel 0.3 --loop 20 --dt-ms 50
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode vel --vel 0.0 --loop 1 --dt-ms 20
```

## 5. 直接用仓库示例脚本

仓库已提供：`bindings/python/examples/robstride_wrapper_demo.py`

### 5.1 ping

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
PYTHONPATH=bindings/python/src LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH} \
python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode ping
```

CLI 对照（1 行）：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode ping
```

### 5.2 读参数

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
PYTHONPATH=bindings/python/src LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH} \
python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode read-param --param-id 0x7019 --param-timeout-ms 1000
```

CLI 对照（1 行）：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode read-param --param-id 0x7019
```

### 5.3 MIT

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
PYTHONPATH=bindings/python/src LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH} \
python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode mit --pos 0 --vel 0 --kp 0.5 --kd 0.2 --tau 0 --loop 40 --dt-ms 50
```

CLI 对照（1 行）：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode mit --pos 0 --vel 0 --kp 0.5 --kd 0.2 --tau 0 --loop 40 --dt-ms 50
```

### 5.4 VEL

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
PYTHONPATH=bindings/python/src LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH} \
python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

CLI 对照（1 行）：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

## 6. 常用动作（每节: 1 个 Python + 1 个 CLI）

### 6.1 扫描

Python 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    for mid in range(1, 128):
        try:
            m = ctrl.add_robstride_motor(mid, 0xFF, "rs-06")
            print("hit", mid, m.robstride_ping())
            m.close()
        except Exception:
            pass
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 127
```

### 6.2 ping

Python 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    print(m.robstride_ping())
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode ping
```

### 6.3 读参数

Python 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    print(m.robstride_get_param_f32(0x7019, 500))
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode read-param --param-id 0x7019
```

### 6.4 写参数（带回读校验）

Python 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    m.robstride_write_param_f32(0x700A, 0.3)
    print(m.robstride_get_param_f32(0x700A, 500))
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode write-param --param-id 0x700A --param-value 0.3
```

### 6.5 改设备 ID（set device id）

Python 示例：

```python
from motorbridge import Controller

OLD_ID = 127
NEW_ID = 126

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(OLD_ID, 0xFF, "rs-06")
    m.robstride_set_device_id(NEW_ID)
    m.store_parameters()  # 建议保存
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --set-motor-id 126 --store 1
```

说明：

- RobStride 当前可修改的是设备 ID（`motor-id`）。
- 本项目里传入的 `feedback-id` 对 RobStride 是主机侧通信 host-id（常用 `0xFF`），不是像 Damiao 那样可单独写入电机的 `MST_ID`。
- 因此在当前 RobStride 接口里，没有“单独修改反馈 ID”的 API。

### 6.6 上位机如何区分回包（Damiao vs RobStride）

Python 示例（推荐以 `motor_id` 作为设备主键）：

```python
# 建议：设备缓存键使用 motor_id（例如 127, 126, ...）
device_key = motor_id
```

CLI 示例（扫描后按 id 建立设备表）：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 127
```

说明：

- Damiao：常见工作流是 `motor-id + feedback-id(MST_ID)` 成对管理。
- RobStride：在本项目实现里，状态/故障回包按电机 `device_id`（即 `motor-id`）归属；`feedback-id` 主要是主机侧 host-id 通信参数。
- 所以上位机统一建议：**以 `motor_id/device_id` 作为设备唯一键**，这样多电机场景最稳。

## 7. 全部封装模式（统一 API）

### 7.1 enable

Python 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    m.enable()
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode enable
```

### 7.2 disable

Python 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    m.disable()
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode disable
```

### 7.3 MIT

Python 示例：

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    ctrl.enable_all()
    m.ensure_mode(Mode.MIT, 1000)
    m.send_mit(0.0, 0.0, 0.5, 0.2, 0.0)
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode mit --pos 0 --vel 0 --kp 0.5 --kd 0.2 --tau 0 --loop 20 --dt-ms 50
```

### 7.4 VEL

Python 示例：

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    ctrl.enable_all()
    m.ensure_mode(Mode.VEL, 1000)
    m.send_vel(0.3)
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode vel --vel 0.3 --loop 20 --dt-ms 50
```

### 7.5 POS_VEL（映射到原生 Position）

Python 示例：

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    ctrl.enable_all()
    m.ensure_mode(Mode.POS_VEL, 1000)
    m.send_pos_vel(1.0, 1.5)  # pos=1.0rad, vlim=1.5rad/s
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode pos-vel --pos 1.0 --vlim 1.5 --loop 1 --dt-ms 20
```

## 8. 全部原生模式（run_mode）

按当前仓库源码，RobStride 原生 `run_mode`（`0x7005`）完整是：

- `0 = MIT`
- `1 = Position`
- `2 = Velocity`

### 8.1 原生 MIT（run_mode=0）

Python 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    m.robstride_write_param_i8(0x7005, 0)
    m.send_mit(0.0, 0.0, 0.5, 0.2, 0.0)
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode write-param --param-id 0x7005 --param-value 0
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode mit --pos 0 --vel 0 --kp 0.5 --kd 0.2 --tau 0 --loop 20 --dt-ms 50
```

### 8.2 原生 Position（run_mode=1）

Python 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    m.robstride_write_param_i8(0x7005, 1)
    m.robstride_write_param_f32(0x7016, 1.0)
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode write-param --param-id 0x7005 --param-value 1
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode write-param --param-id 0x7016 --param-value 1.0
```

### 8.3 原生 Velocity（run_mode=2）

Python 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    m.robstride_write_param_i8(0x7005, 2)
    m.robstride_write_param_f32(0x700A, 0.3)
    m.close()
```

CLI 示例：

```bash
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode write-param --param-id 0x7005 --param-value 2
cargo run -p motor_cli --release -- --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode write-param --param-id 0x700A --param-value 0.3
```

## 9. 常用参数 ID（RobStride）

- `0x7005`：`run_mode`（i8）
- `0x700A`：`spd_ref`（f32）
- `0x7016`：`loc_ref`（f32）
- `0x7019`：`mechPos`（f32）
- `0x701B`：`mechVel`（f32）
- `0x701C`：`VBUS`（f32）

## 10. 常见问题与排查

1. `new_socketcan failed` / `interface is down`
- 检查 `can0` 是否 up，重新执行第 2 节命令。

2. `add_robstride_motor failed` / ping 超时
- 先用扫描确认 ID；优先尝试 `feedback_id=0xFF`。
- 确认终端电阻、CANH/CANL 线序、波特率一致。

3. Python 能 import，但运行时报找不到 `.so`
- 确认 `LD_LIBRARY_PATH` 包含 `target/release`。

4. Python 里 `pos-vel` 仍提示 “not supported for RobStride”
- 通常是 Python 加载了旧版 ABI（例如 `bindings/python/src/motorbridge/lib/libmotor_abi.so`）。
- 先重新编译：`cargo build -p motor_abi --release`
- 再显式指定新版库：
  - `MOTORBRIDGE_LIB=/home/w0x7ce/Downloads/dm_candrive/rust_dm/target/release/libmotor_abi.so`

5. 控制异常或抖动
- 先降低 `vel`/`kp`/`kd`，再逐步增加。
- 保留急停链路（`disable`/断使能）。

## 11. 参考文件

- `bindings/python/src/motorbridge/core.py`
- `bindings/python/src/motorbridge/cli.py`
- `bindings/python/examples/robstride_wrapper_demo.py`
- `motor_cli/ROBSTRIDE_API.zh-CN.md`
- `bindings/python/README.zh-CN.md`
