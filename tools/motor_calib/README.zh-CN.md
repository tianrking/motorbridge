# motor_calib

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan + Damiao 串口桥）

- Linux SocketCAN 直接使用网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- 仅 Damiao 可选串口桥链路：`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`。
- Linux SocketCAN 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。


用于 Damiao 电机的 Rust 标定小工具。

```mermaid
flowchart LR
  SCAN["scan 扫描"] --> HIT["发现在线 ID"]
  HIT --> SET["set-id 改地址"]
  SET --> VERIFY["verify 校验"]
  VERIFY --> DONE["形成可维护地址表"]
```

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

## Windows 实验支持（PCAN-USB）

项目主线仍以 Linux 为主。Windows 支持为实验性能力，当前通过 PEAK PCAN 后端实现。

- 安装 PEAK 驱动与 PCAN-Basic 运行时（`PCANBasic.dll`）。
- Windows 下建议使用 `can0@1000000` 做 1Mbps 验证。

Windows 快速验证（使用 `motor_cli`）：

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4310 --motor-id 0x07 --feedback-id 0x17 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
```
