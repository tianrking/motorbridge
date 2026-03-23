# ros2_bridge

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan）

- Linux 下直接使用 SocketCAN 网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- Linux 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 在 SocketCAN 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。


基于 Python（`rclpy`）+ `motorbridge` Python SDK 的 ROS2 桥接。

```mermaid
flowchart LR
  CMD["/motorbridge/cmd (JSON)"] --> NODE["ros2_bridge 节点"]
  NODE --> SDK["motorbridge Python SDK"]
  SDK --> MOTOR["CAN 总线电机"]
  MOTOR --> SDK
  SDK --> NODE
  NODE --> STATE["/motorbridge/state"]
  NODE --> EVT["/motorbridge/event"]
```

## 功能

- 订阅命令 topic 并执行电机指令
- 周期发布电机状态
- 通过同一命令 topic 支持运维操作：
  - `scan`
  - `set_id`
  - `verify`

## Topics

- 订阅：`/motorbridge/cmd`（`std_msgs/String`，JSON）
- 发布：
  - `/motorbridge/state`（`std_msgs/String`，JSON）
  - `/motorbridge/event`（`std_msgs/String`，JSON）

## 命令 JSON

控制命令示例：

```json
{"op":"enable"}
{"op":"disable"}
{"op":"mit","pos":0.0,"vel":0.0,"kp":20.0,"kd":1.0,"tau":0.0,"continuous":true}
{"op":"pos_vel","pos":3.10,"vlim":1.50,"continuous":true}
{"op":"vel","vel":0.5,"continuous":true}
{"op":"force_pos","pos":0.8,"vlim":2.0,"ratio":0.3,"continuous":true}
```

标定命令示例：

```json
{"op":"scan","start_id":1,"end_id":16,"feedback_base":16,"timeout_ms":100}
{"op":"set_id","old_motor_id":2,"old_feedback_id":18,"new_motor_id":5,"new_feedback_id":21,"store":true,"verify":true}
{"op":"verify","motor_id":5,"feedback_id":21,"timeout_ms":1000}
```

## 运行

前置：

- 已安装 ROS2（`rclpy`、`std_msgs`）
- 可导入 `motorbridge` Python 包

在仓库根目录执行：

```bash
PYTHONPATH=bindings/python/src:$PYTHONPATH \
python3 integrations/ros2_bridge/python/motorbridge_ros2_bridge.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

## 命令发布示例

```bash
ros2 topic pub --once /motorbridge/cmd std_msgs/msg/String '{data: "{\"op\":\"enable\"}"}'
ros2 topic pub --once /motorbridge/cmd std_msgs/msg/String '{data: "{\"op\":\"vel\",\"vel\":0.5,\"continuous\":true}"}'
ros2 topic pub --once /motorbridge/cmd std_msgs/msg/String '{data: "{\"op\":\"scan\",\"start_id\":1,\"end_id\":16}"}'
```

## 说明

- `continuous=true` 表示每个定时周期（`--dt-ms`）都会持续发送该控制命令。
- `enable/disable/scan/set_id/verify` 为一次性操作。
