# RobStride API 与参数参考（完整版）

本页是 `motorbridge` 当前 RobStride 控制、参数读写、以及能力边界的完整实用文档。

> English version: [ROBSTRIDE_API.md](ROBSTRIDE_API.md)

## 1）通用设备参数

| 参数 | 含义 | 常用值 |
|---|---|---|
| `channel` | CAN 接口名 | `can0` |
| `model` | RobStride 型号字符串 | `rs-00`、`rs-06` |
| `motor-id` | 设备 ID | 如 `127` |
| `feedback-id` | 命令帧里的主机/反馈 ID | 常用 `0xFF` |
| `loop` | 周期控制发送次数 | `20`~`100` |
| `dt-ms` | 周期发送间隔 | `20`~`50` |

## 2）`motor_cli` 的 RobStride 模式

当前支持：

- `ping`
- `scan`
- `enable`
- `disable`
- `mit`
- `vel`
- `read-param`
- `write-param`

### 2.1 Ping

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode ping
```

### 2.2 MIT

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0 --loop 40 --dt-ms 50
```

### 2.3 速度模式

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

## 3）扫描与改 ID

### 3.1 扫描

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 255
```

说明：

- 第一阶段：`ping + 参数查询探测`。
- 若全范围无 ping 命中：自动回退到盲探脉冲（观察电机是否转动）。
  - `--manual-vel`（默认 `0.2`）
  - `--manual-ms`（默认 `200`）
  - `--manual-gap-ms`（默认 `200`）

### 3.2 改设备 ID

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 \
  --motor-id 127 --feedback-id 0xFF --set-motor-id 126 --store 1
```

## 4）常用参数 ID

| Param ID | 名称 | 类型 | 含义 |
|---|---|---|---|
| `0x7005` | `run_mode` | `i8` | 控制模式选择 |
| `0x700A` | `spd_ref` | `f32` | 目标速度 |
| `0x7019` | `mechPos` | `f32` | 机械位置 |
| `0x701B` | `mechVel` | `f32` | 机械速度 |
| `0x701C` | `VBUS` | `f32` | 母线电压 |

## 5）参数读写

读参数：

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode read-param --param-id 0x7019
```

写参数：

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode write-param --param-id 0x700A --param-value 0.3
```

Python binding 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    print(m.robstride_ping())
    print(m.robstride_get_param_f32(0x7019, 500))
    m.robstride_write_param_f32(0x700A, 0.3)
    m.close()
```

## 6）协议通信类型覆盖情况

当前 `motorbridge` 对 RobStride 协议通信类型的覆盖：

- 已直接使用：`0(GET_DEVICE_ID)`、`1(OPERATION_CONTROL)`、`3(ENABLE)`、`4(DISABLE)`、`6(SET_ZERO_POSITION)`、`7(SET_DEVICE_ID)`、`17(READ_PARAMETER)`、`18(WRITE_PARAMETER)`、`22(SAVE_PARAMETERS)`
- 已接收解析：`2(OPERATION_STATUS)`、`21(FAULT_REPORT)`
- 协议常量存在但尚未形成高层 API：`23(SET_BAUDRATE)`、`24(ACTIVE_REPORT)`、`25(SET_PROTOCOL)`

## 7）完善空间（差距总结）

当前状态：核心闭环已可用（`scan/ping/mit/vel/读写参数/改ID/设零/存参`）。

可优先增强：

1. CLI 增加更语义化的 `position/current` 快捷命令（当前可用写参数实现，但不直观）。
2. CLI 扫描支持多 feedback-host 候选。
3. 暴露 `SET_BAUDRATE / ACTIVE_REPORT / SET_PROTOCOL` 的高层 API。
4. `FAULT_REPORT` 独立结构化解码输出。

## 8）WS 网关 JSON 示例

```json
{"op":"set_target","vendor":"robstride","channel":"can0","model":"rs-06","motor_id":127,"feedback_id":255}
{"op":"robstride_ping","timeout_ms":200}
{"op":"robstride_read_param","param_id":28697,"type":"f32","timeout_ms":200}
{"op":"robstride_write_param","param_id":28682,"type":"f32","value":0.3,"verify":true}
{"op":"vel","vel":0.3,"continuous":true}
{"op":"mit","pos":0.0,"vel":0.0,"kp":8.0,"kd":0.2,"tau":0.0,"continuous":true}
{"op":"scan","vendor":"robstride","start_id":1,"end_id":255,"feedback_ids":"0xFF,0xFE,0x00","timeout_ms":120}
```

## 9）安全建议

- 先小速度、小循环验证，再逐步增大。
- 压测前先确认 CAN 接线、终端电阻和接口状态。
- 长时间控制前先做 ping/读参验证。
- 始终保留急停路径。
