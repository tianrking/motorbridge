# RobStride API and Parameter Reference (Practical)

This page is a practical reference for RobStride control and parameter operations currently supported by `motorbridge`.

> Chinese version: [rs_api_cn.md](rs_api_cn.md)

## 1) Common Device Parameters

| Parameter | Meaning | Typical value |
|---|---|---|
| `channel` | CAN interface name | `can0` |
| `model` | RobStride model string | `rs-00`, `rs-06` |
| `motor-id` | Device ID | e.g. `127` |
| `feedback-id` | Host/feedback ID used in command frame | typically `0xFF` |
| `loop` | Send cycles for periodic control | `20`~`100` |
| `dt-ms` | Send interval per cycle | `20`~`50` |

## 2) Control Modes

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

## 2.3 Velocity

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

## 3) Frequently Used Parameters

| Param ID | Name | Type | Meaning |
|---|---|---|---|
| `0x7005` | `run_mode` | `u32` | control mode selector |
| `0x700A` | `spd_ref` | `f32` | target velocity |
| `0x7019` | `mechPos` | `f32` | mechanical position |
| `0x701B` | `mechVel` | `f32` | mechanical velocity |
| `0x701C` | `VBUS` | `f32` | bus voltage |

## 4) Parameter Read/Write

Read parameter:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode read-param --param-id 0x7019
```

Write parameter:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode write-param --param-id 0x700A --type f32 --value 0.3 --verify
```

Python binding read/write:

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFF, "rs-06")
    print(m.robstride_ping())
    print(m.robstride_get_param_f32(0x7019, 500))
    m.robstride_write_param_f32(0x700A, 0.3)
    m.close()
```

## 5) Scan and ID Update

Unified scan:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 255
```

Set device ID:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 \
  --motor-id 127 --feedback-id 0xFF --mode set-id --new-id 126 --verify
```

## 6) WS Gateway JSON Examples

```json
{"op":"set_target","vendor":"robstride","channel":"can0","model":"rs-06","motor_id":127,"feedback_id":255}
{"op":"robstride_ping","timeout_ms":200}
{"op":"robstride_read_param","param_id":28697,"type":"f32","timeout_ms":200}
{"op":"robstride_write_param","param_id":28682,"type":"f32","value":0.3,"verify":true}
{"op":"vel","vel":0.3,"continuous":true}
{"op":"mit","pos":0.0,"vel":0.0,"kp":8.0,"kd":0.2,"tau":0.0,"continuous":true}
{"op":"scan","vendor":"robstride","start_id":1,"end_id":255,"feedback_ids":"0xFF,0xFE,0x00","timeout_ms":120}
```

## 7) Safety Notes

- Start with small velocity and short loop count.
- Confirm CAN wiring/termination and interface state before stress tests.
- Prefer ping/read-param verification before long periodic control.
- Keep emergency stop path available.
