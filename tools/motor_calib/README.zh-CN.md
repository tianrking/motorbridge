# motor_calib

用于 Damiao 电机的 Rust 标定小工具。

## 功能

- `scan`：扫描总线在线电机（寄存器探测）
- `set-id`：修改 `ESC_ID`/`MST_ID`，可选回读校验
- `verify`：校验当前 `ESC_ID`/`MST_ID`

## 构建

```bash
cargo build -p motor_calib --release
```

## 使用

```bash
cargo run -p motor_calib -- --help
```

扫描/改ID 前请先配置 CAN：

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

### 扫描

```bash
cargo run -p motor_calib -- scan \
  --channel can0 --model 4310 --start-id 0x01 --end-id 0x10 --timeout-ms 100
```

### 改 ID

```bash
cargo run -p motor_calib -- set-id \
  --channel can0 --model 4310 \
  --motor-id 0x02 --feedback-id 0x12 \
  --new-motor-id 0x05 --new-feedback-id 0x15 \
  --store 1 --verify 1
```

### 校验

```bash
cargo run -p motor_calib -- verify \
  --channel can0 --model 4310 --motor-id 0x05 --feedback-id 0x15
```
