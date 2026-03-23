# motorbridge Python SDK

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan + Damiao 串口桥）

- Linux SocketCAN 直接使用网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- 仅 Damiao 可选串口桥链路：`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`。
- Linux SocketCAN 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。


这是基于 `motor_abi` 的 Python 绑定层。

> English version: [README.md](README.md)

## 范围

- 高层 API: `Controller`、`Motor`、`Mode`
- CLI: `motorbridge-cli`
- Controller 构造入口：
  - `Controller(channel=\"can0\")`（SocketCAN/PCAN 路径）
  - `Controller.from_dm_serial(serial_port=\"/dev/ttyACM0\", baud=921600)`（仅 Damiao 串口桥）
- 厂商入口:
  - Damiao: `add_damiao_motor(...)`
  - MyActuator: `add_myactuator_motor(...)`
  - RobStride: `add_robstride_motor(...)`
  - HighTorque: `add_hightorque_motor(...)`

## 快速开始

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    motor = ctrl.add_damiao_motor(0x01, 0x11, "4340P")
    ctrl.enable_all()
    motor.ensure_mode(Mode.MIT, 1000)
    motor.send_mit(0.0, 0.0, 20.0, 1.0, 0.0)
    print(motor.get_state())
    motor.close()
```

Damiao 串口桥示例：

```python
from motorbridge import Controller, Mode

with Controller.from_dm_serial("/dev/ttyACM1", 921600) as ctrl:
    motor = ctrl.add_damiao_motor(0x04, 0x14, "4310")
    ctrl.enable_all()
    motor.ensure_mode(Mode.MIT, 1000)
    motor.send_mit(0.5, 0.0, 20.0, 1.0, 0.0)
    motor.close()
```

RobStride 快速示例:

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    motor = ctrl.add_robstride_motor(127, 0xFF, "rs-00")
    print(motor.robstride_ping())
    print(motor.robstride_get_param_f32(0x7019))
    motor.close()
```

MyActuator 快速示例:

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    motor = ctrl.add_myactuator_motor(1, 0x241, "X8")
    ctrl.enable_all()
    motor.ensure_mode(Mode.POS_VEL, 1000)
    motor.send_pos_vel(3.1416, 2.0)  # rad / rad/s
    print(motor.get_state())
    motor.close()
```

## CLI 示例

Damiao:

```bash
motorbridge-cli run \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride:

```bash
motorbridge-cli run \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode ping
```

RobStride 读参数:

```bash
motorbridge-cli robstride-read-param \
  --channel can0 --model rs-00 --motor-id 127 --param-id 0x7019 --type f32
```

统一扫描（所有 vendor）:

```bash
motorbridge-cli scan --vendor all --channel can0 --start-id 0x01 --end-id 0xFF
```

通过绑定使用 HighTorque：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    motor = ctrl.add_hightorque_motor(1, 0x01, "hightorque")
    motor.send_mit(3.1416, 0.8, 0.0, 0.0, 0.8)  # kp/kd 参数保留，但协议本身不使用
    motor.request_feedback()
    print(motor.get_state())
    motor.close()
```

通过 Rust CLI 使用 HighTorque：

```bash
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --motor-id 1 --mode read
```

## Windows 实验支持（PCAN-USB）

项目主线仍以 Linux 为主。Windows 支持为实验性能力，当前通过 PEAK PCAN 后端实现。

- 安装 PEAK 驱动与 PCAN-Basic 运行时（`PCANBasic.dll`）。
- Windows 下建议使用 `can0@1000000`（映射到 `PCAN_USBBUS1`，1Mbps）。

建议先用 Rust CLI 做快速验证：

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4310 --motor-id 0x07 --feedback-id 0x17 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
```

Windows 本地 wheel 构建：

```bash
python -m pip install --user wheel
set MOTORBRIDGE_LIB=%CD%\\target\\release\\motor_abi.dll
python -m pip wheel --no-build-isolation bindings/python -w bindings/python/dist
python -m pip install bindings/python/dist/motorbridge-*.whl
```

## 示例程序

- Damiao wrapper 示例: `examples/python_wrapper_demo.py`
- RobStride wrapper 示例: `examples/robstride_wrapper_demo.py`
- Damiao 全模式示例: `examples/full_modes_demo.py`
- Damiao 扫描 / 调参 / 位置辅助:
  - `examples/scan_ids_demo.py`
  - `examples/pid_register_tune_demo.py`
  - `examples/pos_ctrl_demo.py`
  - `examples/pos_repl_demo.py`

详细见 [examples/README.md](examples/README.md)。

## 端到端示例命令

```bash
# 先构建 ABI
cargo build -p motor_abi --release
export PYTHONPATH=bindings/python/src
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}

# Damiao wrapper 示例
python3 bindings/python/examples/python_wrapper_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 20 --dt-ms 20

# RobStride wrapper 示例：ping
python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --mode ping

# RobStride wrapper 示例：速度
python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

## 说明

- `id-dump`、`id-set` 仍是 Damiao 工作流；`scan` 支持 `damiao|myactuator|robstride|hightorque|all`。
- MyActuator 在 ABI wrapper 中不支持 `Mode.MIT` 与 `send_force_pos`。
- Damiao 的完整调参参考仍保留在:
  - [DAMIAO_API.md](DAMIAO_API.md)
  - [DAMIAO_API.zh-CN.md](DAMIAO_API.zh-CN.md)
