# Damiao Python Binding 完整接口参考（`bindings/python`）

本文仅覆盖 **Damiao** 相关 Python binding 接口（`motorbridge`），包含 `dm-serial` 与标准 CAN 的用法。

## 0. 完整性声明（重要）

- 本文档已按 `bindings/python/src/motorbridge/core.py` 中 `Controller` 与 `Motor` 的 **Damiao 可用方法**进行覆盖。
- 若方法属于其他厂商专用（如 `robstride_*`），本文不展开。
- 建议把本文作为 Damiao 路径的“完整接口基准文档”；若后续 SDK 增删接口，请同步更新本文件。

## 1. 适用范围

- 包路径：`bindings/python/src/motorbridge`
- 主要对象：`Controller`、`Motor`、`Mode`、`MotorState`
- 传输：
  - 标准 CAN：`Controller(channel="can0")`
  - Damiao 串口桥：`Controller.from_dm_serial("/dev/ttyACM0", 921600)`

## 2. 基础准备

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
export PYTHONPATH=bindings/python/src
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}
```

## 2.1 参数中文释义总表（常用）

### A) 连接与设备参数

| 参数 | 中文含义 | 常见值/说明 |
|---|---|---|
| `channel` | 标准 CAN 通道名 | `can0` / `can1` / `slcan0` |
| `serial_port` / `--serial-port` | 串口设备路径 | `/dev/ttyACM0` |
| `serial_baud` / `--serial-baud` | 串口波特率 | 常用 `921600` |
| `model` / `--model` | 电机型号提示字符串 | `4310` / `4340P` |
| `motor_id` / `--motor-id` | 目标电机接收 ID（ESC_ID） | 例：`0x07` |
| `feedback_id` / `--feedback-id` | 电机反馈 ID（MST_ID） | 常见 `motor_id + 0x10`，例：`0x17` |

### B) 模式与循环参数

| 参数 | 中文含义 | 常见值/说明 |
|---|---|---|
| `mode` / `--mode` | 控制模式 | `mit` / `pos-vel` / `vel` / `force-pos` / `enable` / `disable` |
| `ensure_mode` / `--ensure-mode` | 控制前是否先切模式 | `1=先切模式`，`0=不切模式` |
| `timeout_ms` / `--ensure-timeout-ms` | 模式切换校验超时（毫秒） | `dm-serial` 常用 `200~500`（推荐 `300`） |
| `loop` / `--loop` | 循环发送次数 | 例：`40` / `200` |
| `dt_ms` / `--dt-ms` | 循环间隔（毫秒） | `dm-serial` 建议 `20` 起步 |

### C) 控制目标参数

| 参数 | 中文含义 | 单位/范围 |
|---|---|---|
| `pos` / `--pos` | 目标位置 | `rad` |
| `vel` / `--vel` | 目标速度 | `rad/s` |
| `kp` / `--kp` | 位置刚度系数（MIT） | 无量纲 |
| `kd` / `--kd` | 速度阻尼系数（MIT） | 无量纲 |
| `tau` / `--tau` | 前馈力矩（MIT） | `Nm` |
| `vlim` / `--vlim` | 速度上限（POS_VEL/FORCE_POS） | `rad/s` |
| `ratio` / `--ratio` | 力矩限幅比例（FORCE_POS） | `0~1` |

### D) 寄存器相关参数

| 参数 | 中文含义 | 说明 |
|---|---|---|
| `rid` | 寄存器 ID | 例：`10` 为 `CTRL_MODE` |
| `value` | 寄存器写入值 | 依据寄存器类型（`u32/f32`） |
| `timeout_ms`（读寄存器） | 单次寄存器读取超时（毫秒） | 例：`300` / `500` |
| `store_parameters()` | 存参（写入非易失） | 建议在参数确认后调用 |

## 3. 数据结构与常量

### 3.1 `Mode`（控制模式）

- `Mode.MIT = 1`
- `Mode.POS_VEL = 2`
- `Mode.VEL = 3`
- `Mode.FORCE_POS = 4`

### 3.2 `MotorState`（反馈状态）

`motor.get_state()` 返回：

- `can_id: int`
- `arbitration_id: int`
- `status_code: int`
- `pos: float`（rad）
- `vel: float`（rad/s）
- `torq: float`（Nm）
- `t_mos: float`（C）
- `t_rotor: float`（C）

说明：`MotorState` **不包含当前模式**，当前模式请读寄存器 `RID_CTRL_MODE=10`。

### 3.3 Damiao 常用寄存器常量

- `RID_CTRL_MODE = 10`
- `RID_MST_ID = 7`
- `RID_ESC_ID = 8`
- `RID_TIMEOUT = 9`
- `RID_PMAX = 21`
- `RID_VMAX = 22`
- `RID_TMAX = 23`
- `RID_KP_ASR = 25`
- `RID_KI_ASR = 26`
- `RID_KP_APR = 27`
- `RID_KI_APR = 28`

## 4. `Controller` 接口（Damiao 相关）

### 4.1 构造

- `Controller(channel: str = "can0")`
- `Controller.from_dm_serial(serial_port: str = "/dev/ttyACM0", baud: int = 921600)`

### 4.2 生命周期 / 总线控制

- `close()`
- `shutdown()`
- `close_bus()`
- `__enter__() -> Controller`
- `__exit__(exc_type, exc, tb) -> None`

### 4.3 广播控制

- `enable_all()`
- `disable_all()`
- `poll_feedback_once()`

### 4.4 添加电机

- `add_damiao_motor(motor_id: int, feedback_id: int, model: str) -> Motor`

> 说明：`add_myactuator_motor/add_robstride_motor/add_hightorque_motor` 为跨厂商入口，不属于 Damiao 专项参考范围。

## 5. `Motor` 接口（Damiao 相关）

### 5.1 基础动作

- `enable()`
- `disable()`
- `clear_error()`
- `set_zero_position()`
- `close()`

### 5.2 模式切换

- `ensure_mode(mode: Mode, timeout_ms: int = 1000)`

说明：

- `timeout_ms` 是模式切换校验超时（毫秒）
- 常用建议：
  - 标准 CAN：`150~300`
  - `dm-serial`：`200~500`（常用 `300`）

### 5.3 控制命令

- `send_mit(pos: float, vel: float, kp: float, kd: float, tau: float)`
- `send_pos_vel(pos: float, vlim: float)`
- `send_vel(vel: float)`
- `send_force_pos(pos: float, vlim: float, ratio: float)`

单位：

- `pos`: rad
- `vel` / `vlim`: rad/s
- `tau`: Nm
- `ratio`: 0~1

### 5.4 反馈与状态

- `request_feedback()`
- `get_state() -> MotorState | None`

### 5.5 寄存器

- `get_register_u32(rid: int, timeout_ms: int = 1000) -> int`
- `get_register_f32(rid: int, timeout_ms: int = 1000) -> float`
- `write_register_u32(rid: int, value: int)`
- `write_register_f32(rid: int, value: float)`
- `store_parameters()`
- `set_can_timeout_ms(timeout_ms: int)`

## 5.6 Damiao 全方法签名清单（对照 `core.py`）

### Controller（Damiao 使用时）

- `Controller(channel: str = "can0")`
- `Controller.from_dm_serial(serial_port: str = "/dev/ttyACM0", baud: int = 921600) -> Controller`
- `close() -> None`
- `shutdown() -> None`
- `close_bus() -> None`
- `enable_all() -> None`
- `disable_all() -> None`
- `poll_feedback_once() -> None`
- `add_damiao_motor(motor_id: int, feedback_id: int, model: str) -> Motor`
- `__enter__() -> Controller`
- `__exit__(exc_type, exc, tb) -> None`

### Motor（Damiao 使用时）

- `close() -> None`
- `enable() -> None`
- `disable() -> None`
- `clear_error() -> None`
- `set_zero_position() -> None`
- `ensure_mode(mode: Mode, timeout_ms: int = 1000) -> None`
- `send_mit(pos: float, vel: float, kp: float, kd: float, tau: float) -> None`
- `send_pos_vel(pos: float, vlim: float) -> None`
- `send_vel(vel: float) -> None`
- `send_force_pos(pos: float, vlim: float, ratio: float) -> None`
- `request_feedback() -> None`
- `set_can_timeout_ms(timeout_ms: int) -> None`
- `store_parameters() -> None`
- `write_register_f32(rid: int, value: float) -> None`
- `write_register_u32(rid: int, value: int) -> None`
- `get_register_f32(rid: int, timeout_ms: int = 1000) -> float`
- `get_register_u32(rid: int, timeout_ms: int = 1000) -> int`
- `get_state() -> MotorState | None`

## 5.7 方法说明（作用 / 联动调用 / 参数范围）

### A) Controller 方法说明

| 方法 | 作用 | 常见联动调用 | 参数范围/建议 |
|---|---|---|---|
| `Controller(channel)` | 走标准 CAN 创建控制器 | `add_damiao_motor` | `channel` 例：`can0` |
| `Controller.from_dm_serial(serial_port, baud)` | 走 Damiao 串口桥创建控制器 | `add_damiao_motor` | `serial_port` 例：`/dev/ttyACM0`；`baud` 常用 `921600` |
| `add_damiao_motor(motor_id, feedback_id, model)` | 绑定一个 Damiao 电机句柄 | `enable_all` / `ensure_mode` / `send_xxx` | `motor_id` 常见 `0x01~0x7F`；`feedback_id` 常见 `motor_id+0x10`；`model` 如 `4310/4340P` |
| `enable_all()` | 广播使能当前控制器已添加电机 | `ensure_mode` 前调用 | 建议后接 `100~300ms` 稳定时间 |
| `disable_all()` | 广播失能 | 结束流程前调用 | 无硬性范围 |
| `poll_feedback_once()` | 手动消费一次接收队列并更新状态缓存 | 常与 `request_feedback` 连用 | 非必需（有后台轮询时）；用于“立即刷新” |
| `shutdown()` | 关闭控制器并执行收尾（含设备处理） | `with` 退出时自动执行 | 推荐正常退出使用 |
| `close_bus()` | 仅关闭总线层 | 特殊场景手动管理生命周期 | 不会替代完整 shutdown 语义 |
| `close()` | 释放 Python 侧句柄 | `shutdown` 后调用 | `with` 退出时自动调用 |

### B) Motor 方法说明

| 方法 | 作用 | 常见联动调用 | 参数范围/建议 |
|---|---|---|---|
| `enable()` | 单电机使能 | `clear_error` 之后、控制前 | 建议后接短等待（`100~300ms`） |
| `disable()` | 单电机失能 | 流程结束 | 无硬性范围 |
| `clear_error()` | 清故障状态 | `enable/ensure_mode` 前 | 建议维护流程先调用 |
| `set_zero_position()` | 将“当前位置”设为零点 | **项目规范：先 `disable` 再调用** | 不是“回到 0 位”；不建议每次运行都调用 |
| `ensure_mode(mode, timeout_ms)` | 切换并校验控制模式 | `send_xxx` 前 | `mode`=`MIT/POS_VEL/VEL/FORCE_POS`；`timeout_ms` 建议：标准 CAN `150~300`，`dm-serial` `200~500` |
| `send_mit(pos, vel, kp, kd, tau)` | MIT 控制 | `ensure_mode(Mode.MIT)` 后 | `pos` rad；`vel` rad/s；`kp>=0`、`kd>=0`；`tau` Nm。建议小参数起步 |
| `send_pos_vel(pos, vlim)` | 位置+速度上限控制 | `ensure_mode(Mode.POS_VEL)` 后 | `pos` rad；`vlim>0`，建议从 `0.5~2.0` 起步 |
| `send_vel(vel)` | 速度控制 | `ensure_mode(Mode.VEL)` 后 | `vel` rad/s，建议从小值起步 |
| `send_force_pos(pos, vlim, ratio)` | 位置+力矩限幅 | `ensure_mode(Mode.FORCE_POS)` 后 | `ratio` 推荐 `0~1`（常用 `0.1~0.5`） |
| `request_feedback()` | 主动请求一帧反馈 | `poll_feedback_once` / `get_state` | 常用于需要“当前新鲜状态”的场景 |
| `get_state()` | 读取最近缓存状态 | `request_feedback` 后读取更稳 | 可能返回 `None`（尚未收到反馈） |
| `set_can_timeout_ms(timeout_ms)` | 设置电机 CAN 超时参数 | 维护流程 | `timeout_ms>0`，常用 `100~2000` |
| `store_parameters()` | 存参（写入非易失） | 修改寄存器后按需调用 | 建议“确认值正确后”再存 |
| `write_register_u32(rid, value)` | 写 u32 寄存器 | 可选 `store_parameters` | `rid` 必须是可写且 u32 类型 |
| `write_register_f32(rid, value)` | 写 f32 寄存器 | 可选 `store_parameters` | `rid` 必须是可写且 f32 类型 |
| `get_register_u32(rid, timeout_ms)` | 读 u32 寄存器 | 诊断/模式确认 | `timeout_ms` 常用 `300~1000`；太小易超时 |
| `get_register_f32(rid, timeout_ms)` | 读 f32 寄存器 | 诊断/参数核对 | `timeout_ms` 常用 `300~1000` |
| `close()` | 释放电机句柄 | 结束时调用 | `with` 场景仍建议显式关闭 motor |

### C) 标准联动顺序（建议）

1. `Controller.from_dm_serial(...)`
2. `add_damiao_motor(...)`
3. 维护预处理：`clear_error()`（可选 `set_zero_position()`）
4. `enable_all()`（或 `enable()`）
5. `ensure_mode(...)`
6. `send_xxx(...)` 循环
7. 需要新鲜状态时：`request_feedback()` + `poll_feedback_once()` + `get_state()`
8. 结束：`disable()`/`disable_all()` + `shutdown()/close()`

## 5.8 `set_zero_position()` 时序说明（dm-serial 重点）

### A) 命令语义

- `set_zero_position()` 是“把当前位置设为零点参考”，不是“让电机转回 0”。
- 底层发送的是 Damiao 置零命令帧（data `FF FF FF FF FF FF FF FE`）。

### B) 是否必须等待

- 协议层面：不是硬性必须等待。
- 工程实践（dm-serial）：建议在 `set_zero_position()` 后保留短暂稳定时间，再做 `ensure_mode` 或寄存器读取。
- 已有实测结论：`set_zero` 后紧接 `ensure_mode` 可能触发 `register 10 not received`；增加约 `20 ms` 等待后可显著稳定。

### B.1) 项目约束（重点）

- 在本项目 dm-serial 实操规范中，`set_zero_position()` 前 **必须先 `disable`**。
- 建议按“强约束流程”执行：`disable -> set_zero_position -> wait(~20ms) -> enable -> ensure_mode -> control`。
- 原因：可显著降低 `set_zero` 后寄存器读超时（如 `RID 10`）导致的后续控制失败风险。

### C) 推荐顺序（校准后要继续控制时）

1. `disable`（或 `disable_all`）
2. `set_zero_position`
3. 等待 `~20 ms`（若现场不稳可放宽到 `50~200 ms`）
4. `enable`（或 `enable_all`）
5. `ensure_mode`
6. `send_xxx` 控制

### D) 触发异常后的软件恢复（不重启优先）

- 建议顺序：`disable -> clear_error -> enable -> 重试 ensure_mode`
- 如果仍失败，先 `scan` 确认当前在线 ID（避免把 `0x07/0x17` 当成 `0x04/0x14`）。
- 若反馈帧有但寄存器读持续失败，再考虑设备侧重新上电。

## 6. 常用读写模式（推荐）

### 6.1 查询“当前模式 + 实时状态”

```python
from motorbridge import Controller, RID_CTRL_MODE

with Controller.from_dm_serial("/dev/ttyACM0", 921600) as ctrl:
    m = ctrl.add_damiao_motor(0x07, 0x17, "4310")
    m.request_feedback()
    ctrl.poll_feedback_once()
    st = m.get_state()
    mode = m.get_register_u32(RID_CTRL_MODE, 300)  # 1/2/3/4
    print("mode=", mode, "state=", st)
    m.close()
```

### 6.2 推荐控制流程

1. `enable_all()`
2. `ensure_mode(...)`
3. `send_xxx(...)`
4. `request_feedback()` + `poll_feedback_once()` + `get_state()`
5. 结束时 `disable()` 或 `disable_all()`

## 7. Python CLI（`python -m motorbridge.cli`）Damiao 子集

### 7.1 扫描

```bash
python3 -m motorbridge.cli scan \
  --vendor damiao --transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600 \
  --model 4310 --start-id 0x01 --end-id 0x10
```

### 7.2 控制

```bash
python3 -m motorbridge.cli run \
  --vendor damiao --transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600 \
  --model 4310 --motor-id 0x07 --feedback-id 0x17 \
  --mode mit --ensure-mode 1 --ensure-timeout-ms 300 \
  --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0 --loop 40 --dt-ms 20
```

`run --mode` 支持：

- `enable`
- `disable`
- `mit`
- `pos-vel`
- `vel`
- `force-pos`

### 7.3 ID/寄存器辅助命令

- `id-dump`（读取 ID/模式/关键寄存器）
- `id-set`（写入 ESC_ID / MST_ID，可选 store+verify）

## 8. `dm-serial` 实战建议

- `dm-serial` 仅用于 `--vendor damiao`
- 先扫描再控制，ID 必须匹配
- 切模式建议 `ensure_mode=1`
- 高频下更易抖动，建议从 `dt-ms=20` 起步
- 若模式切换异常，先 `disable -> clear_error -> enable -> ensure_mode` 再继续

## 9. 现成示例脚本（Damiao）

- `bindings/python/examples/damiao_dm_serial_demo.py`
- `bindings/python/examples/dm_serial_mode_switch_200_demo.py`
- `bindings/python/examples/dm_serial_status_like_cli_demo.py`
- `bindings/python/examples/scan_ids_demo.py`（标准 CAN 路径）

---

如需“只保留 dm-serial 的极简接口版参考”，可在此文档基础上再裁剪一版 `dm_serial_only` 速查表。
