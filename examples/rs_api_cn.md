# RobStride API 与参数参考（实用版）

本页是 `motorbridge` 当前 RobStride 控制与参数操作的实用总表。

> English version: [rs_api.md](rs_api.md)
> 详细能力对照审计（含改进空间）: [rs_api_gap_analysis.zh-CN.md](rs_api_gap_analysis.zh-CN.md)

## 1）通用设备参数

| 参数 | 含义 | 常用值 |
|---|---|---|
| `channel` | CAN 接口名 | `can0` |
| `model` | RobStride 型号字符串 | `rs-00`、`rs-06` |
| `motor-id` | 设备 ID | 如 `127` |
| `feedback-id` | 命令帧里的主机/反馈 ID | 常用 `0xFF` |
| `loop` | 周期控制发送次数 | `20`~`100` |
| `dt-ms` | 周期发送间隔 | `20`~`50` |

## 2）控制模式

## 2.1 Ping

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode ping
```

## 2.2 MIT

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0 --loop 40 --dt-ms 50
```

## 2.3 速度模式

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

## 3）常用参数 ID

| Param ID | 名称 | 类型 | 含义 |
|---|---|---|---|
| `0x7005` | `run_mode` | `i8` | 控制模式选择 |
| `0x700A` | `spd_ref` | `f32` | 目标速度 |
| `0x7019` | `mechPos` | `f32` | 机械位置 |
| `0x701B` | `mechVel` | `f32` | 机械速度 |
| `0x701C` | `VBUS` | `f32` | 母线电压 |

## 4）参数读写

读参数：

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode read-param --param-id 0x7019
```

写参数：

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode write-param --param-id 0x700A --param-value 0.3
```

Python binding 读写示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    print(m.robstride_ping())
    print(m.robstride_get_param_f32(0x7019, 500))
    m.robstride_write_param_f32(0x700A, 0.3)
    m.close()
```

## 5）扫描与改 ID

扫描：

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 255
```

改设备 ID：

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 \
  --motor-id 127 --feedback-id 0xFF --set-motor-id 126 --store 1
```

## 6）WS 网关 JSON 示例

```json
{"op":"set_target","vendor":"robstride","channel":"can0","model":"rs-06","motor_id":127,"feedback_id":255}
{"op":"robstride_ping","timeout_ms":200}
{"op":"robstride_read_param","param_id":28697,"type":"f32","timeout_ms":200}
{"op":"robstride_write_param","param_id":28682,"type":"f32","value":0.3,"verify":true}
{"op":"vel","vel":0.3,"continuous":true}
{"op":"mit","pos":0.0,"vel":0.0,"kp":8.0,"kd":0.2,"tau":0.0,"continuous":true}
{"op":"scan","vendor":"robstride","start_id":1,"end_id":255,"feedback_ids":"0xFF,0xFE,0x00","timeout_ms":120}
```

## 7）安全建议

- 先小速度、小循环验证，再逐步增大。
- 压测前先确认 CAN 接线、终端电阻和接口状态。
- 长时间控制前先做 ping / 读参验证链路。
- 保持急停与机械安全余量。
