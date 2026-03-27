# C ABI 示例

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan + CAN-FD + Damiao 串口桥）

- Linux SocketCAN 直接使用网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- Hexfellow 示例需使用 CAN-FD 路径（`motor_controller_new_socketcanfd(...)` / CLI `--transport socketcanfd`）。
- 仅 Damiao 可选串口桥链路：`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`。
- Damiao 串口桥完整接口与命令模板见 `motor_cli/README.zh-CN.md` 第 `3.6` 节（英文见 `motor_cli/README.md`）。
- Linux SocketCAN 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。


这里是直接调用 `motor_abi` 的 C 示例。

> English version: [README.md](README.md)

## 文件

- `c_abi_demo.c`: Damiao + RobStride 的统一示例
- `hexfellow_canfd_demo.c`: Hexfellow CAN-FD 示例（仅 `mit` / `pos-vel`）

覆盖范围:

- Damiao: `enable`、`disable`、`mit`、`pos-vel`、`vel`、`force-pos`
- RobStride: `ping`、`enable`、`disable`、`mit`、`vel`、`read-param`、`write-param`
- Hexfellow（仅 CAN-FD）: `mit`、`pos-vel`

## 构建

```bash
cargo build -p motor_abi --release
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
cc examples/c/hexfellow_canfd_demo.c -I motor_abi/include -L target/release -lmotor_abi -o hexfellow_canfd_demo
LD_LIBRARY_PATH=target/release ./c_abi_demo --help
```

## 示例

Damiao MIT:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride ping:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride 读取位置参数:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019 --param-type f32
```

RobStride 低增益 MIT:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0 --loop 20 --dt-ms 50
```

Hexfellow（仅 CAN-FD）:

```bash
LD_LIBRARY_PATH=target/release ./hexfellow_canfd_demo \
  --channel can0 --motor-id 0x01 --feedback-id 0x00 --mode mit --loop 20 --dt-ms 50
```
