# CLI 指南（`motor_cli`）

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan + Damiao 串口桥）

- Linux SocketCAN 直接使用网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- 仅 Damiao 可选独立 CAN-FD 链路：`--transport socketcanfd`（与经典 `socketcan` 并存）。
- 仅 Damiao 可选串口桥链路：`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`。
- Linux SocketCAN 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。

传输标识：
- `[STD-CAN]` => `--transport auto|socketcan`
- `[CAN-FD]` => `--transport socketcanfd`
- `[DM-SERIAL]` => `--transport dm-serial`

`[CAN-FD]` 说明：目前是“链路已接入”，电机验证矩阵尚未声明完成。

## 调试入口

- Linux `slcan` + Windows `pcan` 专业排障见：[can_debugging.md](can_debugging.md)。

## 构建

```bash
cargo build -p motor_cli --release
```

## 通用参数

- `--vendor damiao|robstride|hightorque|myactuator|hexfellow|all`
- `--transport auto|socketcan|socketcanfd|dm-serial`（`socketcanfd` 与 `dm-serial` 仅 Damiao）
- `--channel can0`
- `--serial-port /dev/ttyACM0 --serial-baud 921600`（配合 `--transport dm-serial`）
- `--motor-id <id>`
- `--loop <n> --dt-ms <ms>`

## Damiao 示例

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```
`[STD-CAN]`

```bash
# Damiao 串口桥链路
cargo run -p motor_cli --release -- \
  --vendor damiao --transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600 \
  --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --verify-model 0 --ensure-mode 0 \
  --pos 0.5 --vel 0 --kp 20 --kd 1 --tau 0 --loop 80 --dt-ms 20
```
`[DM-SERIAL]`

```bash
# Damiao 独立 CAN-FD 链路
cargo run -p motor_cli --release -- \
  --vendor damiao --transport socketcanfd --channel can0 \
  --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --verify-model 0 --ensure-mode 0 \
  --pos 0.5 --vel 0 --kp 20 --kd 1 --tau 0 --loop 80 --dt-ms 20
```
`[CAN-FD]`

## Hexfellow 示例

```bash
# Hexfellow 扫描（CAN-FD 链路）
cargo run -p motor_cli --release -- \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --mode scan --start-id 1 --end-id 32
```
`[CAN-FD]`

```bash
# Hexfellow 状态查询
cargo run -p motor_cli --release -- \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --model hexfellow --motor-id 1 --feedback-id 0 \
  --mode status
```
`[CAN-FD]`

## RobStride 示例

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0 --loop 20 --dt-ms 50
```

## HighTorque（原生 `ht_can` v1.5.5）

支持模式：

- `scan`
- `read` / `ping`
- `mit`（统一接口）
- `pos` / `vel` / `tqe`
- `pos-vel-tqe`
- `volt` / `cur`
- `stop` / `brake` / `rezero` / `conf-write` / `timed-read`

统一单位接口（与其他电机保持一致）：

- `--pos`：弧度（rad）
- `--vel`：弧度每秒（rad/s）
- `--tau`：扭矩（Nm）
- `--kp`、`--kd`：为统一 MIT 参数签名保留，`ht_can` 协议本身不使用

底层原始接口（调试）：

- `--raw-pos`、`--raw-vel`、`--raw-tqe`

示例：

```bash
# 扫描 ID
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --mode scan --start-id 1 --end-id 32
```

```bash
# 读状态（输出包含 pos_rad / vel_rad_s）
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --motor-id 1 --mode read
```

```bash
# 转到 +180 度（pi 弧度），并限制速度/力矩
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --motor-id 1 \
  --mode mit --pos 3.1415926 --vel 0.8 --tau 0.8
```

```bash
# 停止
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --motor-id 1 --mode stop
```

## MyActuator 示例

```bash
cargo run -p motor_cli --release -- \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode status --loop 20 --dt-ms 50
```

```bash
# 将当前位置设为零点（持久生效需断电重启）
cargo run -p motor_cli --release -- \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode set-zero --loop 1
```

## 全品牌扫描

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```
