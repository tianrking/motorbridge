# MyActuator API 与模式参考（完整版）

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan）

- Linux 下直接使用 SocketCAN 网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- Linux 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 在 SocketCAN 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。


本页是 `motor_cli` 中 MyActuator 控制能力的实用总表。

## 1）基础信息

- 厂商名：`myactuator`
- CAN ID 规则：
  - 发送命令：`0x140 + motor_id`
  - 接收反馈：`0x240 + motor_id`
- 常见 ID 范围：`motor_id in [1, 32]`
- 对于 ID=1 的默认值：
  - `--motor-id 1`
  - `--feedback-id 0x241`

## 2）`motor_cli` 支持模式

- `scan`：扫描 1..32
- `enable`：释放刹车（`0x77`）
- `disable`：关电机（`0x80`）
- `stop`：停止闭环（`0x81`）
- `set-zero`：将当前位置写为编码器零点（`0x64`，断电重启后持久生效）
- `status`：读取状态2（`0x9C`）
- `current`：电流闭环（`0xA1`）
- `vel`：速度闭环（`0xA2`）
- `pos`：绝对位置闭环（`0xA4`）
- `version`：读取版本日期（`0xB2`）
- `mode-query`：读取系统运行模式（`0x70`）

## 3）参数与单位

| 参数 | 类型 | 默认值 | 作用模式 | 单位 |
|---|---|---|---|---|
| `--start-id` | u16 | `1` | `scan` | id |
| `--end-id` | u16 | `32` | `scan` | id |
| `--current` | f32 | `0.0` | `current` | A |
| `--vel` | f32 | `0.0` | `vel` | rad/s |
| `--pos` | f32 | `0.0` | `pos` | rad |
| `--max-speed` | f32 | `8.726646` | `pos` | rad/s |
| `--loop` | u64 | `1` | 全部 | 次 |
| `--dt-ms` | u64 | `20` | 全部 | ms |

## 4）当前 CLI 的编码细节

CLI 输入统一使用弧度/弧度每秒，再在内部转换为 MyActuator 协议使用的角度/角度每秒。

- `A1` 电流模式：
  - `[4..5] = int16(current / 0.01)`
- `A2` 速度模式：
  - `[4..7] = int32((vel_rad_s.to_degrees()) * 100)`
- `A4` 绝对位置模式：
  - `[2..3] = uint16(max_speed_rad_s.to_degrees())`
  - `[4..7] = int32(pos_rad.to_degrees() * 100)`

## 5）反馈解码（状态/命令回包）

对于 `0x9C`、`0xA1`、`0xA2`、`0xA4` 回包：

- `data[1]`：温度（int8, °C）
- `data[2..3]`：电流（int16 * 0.01 A）
- `data[4..5]`：速度（int16, deg/s）
- `data[6..7]`：轴角（int16 * 0.01 deg，近圈角）

对于 `0x92` 多圈角回包：

- `data[4..7]`：多圈角（int32 * 0.01 deg）

CLI 的状态输出现在同时打印：

- `angle=...`：来自状态2（`0x9C`）
- `mt_angle=...`：来自多圈角（`0x92`），用于绝对位置判定

## 6）常用命令示例

```bash
# 扫描 MyActuator ID
motor_cli --vendor myactuator --channel can0 --mode scan --start-id 1 --end-id 32

# 连续状态读取
motor_cli --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode status --loop 40 --dt-ms 50

# 速度控制（+0.5236 rad/s ~= +30 deg/s）
motor_cli --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode vel --vel 0.5236 --loop 100 --dt-ms 20

# 位置控制（pi rad = 180 度，最大 5.236 rad/s ~= 300 deg/s）
motor_cli --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode pos --pos 3.1416 --max-speed 5.236 --loop 1

# 将当前位置设为零点（持久生效需要断电重启）
motor_cli --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode set-zero --loop 1
```

## 7）故障排查

如果“有回包但不转”：

1. 用 `0x9A` 查询状态1并看故障码。 
2. 若故障码为 `0x0004`，表示**欠压保护**。 
3. 先恢复供电电压，再执行复位（`0x76`）与释放刹车（`0x77`）。

状态1常用字段：

- `data[3]`：刹车释放标志
- `data[4..5]`：母线电压（`uint16 * 0.1 V`）
- `data[6..7]`：故障码
