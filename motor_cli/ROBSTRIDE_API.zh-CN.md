# RobStride API 与参数参考（完整版）

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan + Damiao 串口桥）

- Linux SocketCAN 直接使用网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- 仅 Damiao 可选串口桥链路：`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`。
- Linux SocketCAN 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。


本页是 `motorbridge` 当前 RobStride 控制、参数读写、以及能力边界的完整实用文档。

> English version: [ROBSTRIDE_API.md](ROBSTRIDE_API.md)

## 0）运行方式速查（不必每次 `cargo run`）

`motor_cli` 有三种常见运行方式：

1. Release 二进制（推荐现场）
```bash
./bin/motor_cli --vendor robstride --mode scan --start-id 1 --end-id 255
```

2. 源码方式（开发调试）
```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 255
```

3. Python 包安装后（不编译 Rust）
```bash
python3 -m pip install motorbridge
motorbridge-cli scan --vendor robstride --channel can0 --model rs-06 --start-id 1 --end-id 255
```

说明：
- 本文命令默认写成 `motor_cli ...` 形式；可等价替换为上面三种入口之一。
- 如果你已 `pip install motorbridge`，优先用 `motorbridge-cli` 即可，不需要每次 `cargo run`。
- 使用源码目录中的 Python CLI 时，建议加：
  `LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}`（确保加载当前仓库 ABI 动态库）。

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
- `pos-vel`
- `vel`
- `read-param`
- `write-param`

大一统“四协议”映射状态：

| 大一统能力 | RobStride 状态 | 说明 |
|---|---|---|
| `MIT` | 已支持 | 原生 operation-control 帧 |
| `POS_VEL` | 已支持 | 映射到 `run_mode=1` + `0x7017/0x7016` |
| `VEL` | 已支持 | 映射到 `run_mode=2` + `0x700A` |
| `TORQUE/CURRENT` | 仅参数级 | 尚无统一高层模式；通过 `write-param` 写 `iq_ref`/限幅参数 |

### 2.1 Ping

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode ping
```

### 2.2 MIT

MIT 映射说明（统一接口 -> RobStride 原生）：

- 有效参数：`--pos`、`--vel`、`--kp`、`--kd`、`--tau`（五个都生效）。
- 单位约定：
  - `--pos`：`rad`
  - `--vel`：`rad/s`
  - `--tau`：`Nm`
  - `--kp`、`--kd`：MIT 闭环增益

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode mit --pos 0 --vel 0 --kp 0.5 --kd 0.2 --tau 0 --loop 40 --dt-ms 50
```

### 2.3 速度模式

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

### 2.4 位置模式（统一 `pos-vel` 映射）

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode pos-vel --pos 1.0 --vlim 1.5 --loop 1 --dt-ms 20
```

说明：

- 统一 `pos-vel` 已映射为 RobStride 原生 Position 链路：
  - `run_mode=1`（Position）
  - 写 `0x7017`（`limit_spd`）为 `--vlim`
  - 可选写 `0x701E`（`loc_kp`）来自 `--loc-kp` 或 `--kp`
  - 写 `0x7016`（`loc_ref`）为 `--pos`
- `--vel`、`--kd`、`--tau` 不属于原生 Position 模式，在 `--mode pos-vel` 下会被忽略。

### 2.5 两种使用方式（统一封装 / 原生参数）

- 统一封装方式（推荐上层业务使用）：
  - `--mode mit`
  - `--mode pos-vel`（已映射到原生 Position）
  - `--mode vel`
- 原生方式（调试/协议级验证）：
  - `--mode read-param --param-id ...`
  - `--mode write-param --param-id ... --param-value ...`
  - 典型链路：先写 `run_mode(0x7005)`，再写对应目标参数（如 `loc_ref/spd_ref`）

## 3）扫描与改 ID

### 3.1 扫描

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 255
```

联调建议（更快）：

```bash
motorbridge-cli scan \
  --vendor robstride --channel can0 --model rs-06 \
  --start-id 120 --end-id 130 --feedback-ids 0xFD --param-timeout-ms 60
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
  --motor-id 127 --feedback-id 0xFD --set-motor-id 126 --store 1
```

与上位机抓包对齐的报文说明：

- 改 ID 使用 `comm_type=7`。
- 该路径下扩展 ID 组成为：
  - `0x07 [new_id] [host_id] [old_id]`
  - 例如（`old_id=1`、`new_id=11`、`host_id=0xFD`）：`0x070BFD01`
- 数据区优先使用最近一次 `ping` 的 UUID token（若拿不到 token 则回退为全 0）。

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
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode read-param --param-id 0x7019
```

写参数：

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode write-param --param-id 0x700A --param-value 0.3
```

Python binding 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
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

当前状态：核心闭环已可用（`scan/ping/mit/pos-vel/vel/读写参数/改ID/设零/存参`）。

当前已知问题（实测）：

1. `pos-vel` 参数生效性在部分固件上不稳定：
   - `--vlim`（`0x7017`）和 `--kp`/`loc_kp`（`0x701E`）可能回读正常，但体感效果弱或不明显。
   - 当前 `MIT` 路径相对更稳定。
2. RobStride 零点校准仍未稳定：
   - 实验性 `zero` 时序可能发送/ACK 正常，但设备侧 `zero_sta`/`mechPos` 校验仍可能失败。
   - 在完成固件级时序完全对齐前，零点校准视为未彻底解决。

可优先增强：

1. CLI 增加更语义化的 `current/torque` 快捷命令（当前可用写参数实现，但不直观）。
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
{"op":"mit","pos":0.0,"vel":0.0,"kp":0.5,"kd":0.2,"tau":0.0,"continuous":true}
{"op":"scan","vendor":"robstride","start_id":1,"end_id":255,"feedback_ids":"0xFD,0xFF,0xFE","timeout_ms":120}
```

## 9）安全建议

- 先小速度、小循环验证，再逐步增大。
- 压测前先确认 CAN 接线、终端电阻和接口状态。
- 长时间控制前先做 ping/读参验证。
- 始终保留急停路径。



