# CLI 指南（`motor_cli`）

## 构建

```bash
cargo build -p motor_cli --release
```

## 通用参数

- `--vendor damiao|robstride|hightorque|myactuator|all`
- `--channel can0`
- `--motor-id <id>`
- `--loop <n> --dt-ms <ms>`

## Damiao 示例

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

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

## 全品牌扫描

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```
