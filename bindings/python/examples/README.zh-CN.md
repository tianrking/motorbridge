# Python 实用示例

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan + Damiao 串口桥）

- Linux SocketCAN 直接使用网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- 仅 Damiao 可选串口桥链路：`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`。
- Damiao 串口桥完整接口与命令模板见 `motor_cli/README.zh-CN.md` 第 `3.6` 节（英文见 `motor_cli/README.md`）。
- Linux SocketCAN 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。


这里是基于 Python SDK 的示例集合。

> English version: [README.md](README.md)

## 文件

- `python_wrapper_demo.py`: 最小 Damiao MIT 示例
- `robstride_wrapper_demo.py`: RobStride 的 ping / read-param / mit / vel 示例
- `full_modes_demo.py`: Damiao 全模式示例
- `pid_register_tune_demo.py`: Damiao 寄存器调参
- `scan_ids_demo.py`: Damiao 快速扫描（历史辅助脚本）
- `pos_ctrl_demo.py`: Damiao 单次目标位置
- `pos_repl_demo.py`: Damiao 交互式位置控制台

## 快速运行

Damiao:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/python_wrapper_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 20 --dt-ms 20
```

RobStride:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride 读参数:

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-00 --motor-id 127 --mode read-param --param-id 0x7019
```

通过 CLI 做统一扫描:

```bash
PYTHONPATH=bindings/python/src python3 -m motorbridge.cli scan \
  --vendor all --channel can0 --start-id 0x01 --end-id 0xFF
```
