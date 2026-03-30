# Python 实用示例（中文）

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan + CAN-FD + Damiao 串口桥）

- Linux SocketCAN 直接使用接口名：`can0`、`can1`、`slcan0`。
- USB-串口类 CAN 适配器需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- Hexfellow 示例必须使用 CAN-FD 链路（`Controller.from_socketcanfd(...)` / CLI `--transport socketcanfd`）。
- Damiao 可选串口桥链路（CLI）：`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`。
- Damiao 串口桥完整命令模板见 `motor_cli/README.md`（中文见 `motor_cli/README.zh-CN.md` 第 `3.6` 节）。
- Linux SocketCAN 下 `--channel` 不要附加 `@bitrate`（例如 `can0@1000000` 无效）。
- Windows（PCAN 后端）里，`can0/can1` 映射到 `PCAN_USBBUS1/2`，支持可选 `@bitrate` 后缀。

基于 Python SDK 的示例集合。

> English version: [README.md](README.md)

## 新手优先（最简单的 2 个示例）

如果你是第一次上手，建议先看并先跑这两个文件：

- `simple_01_motor_control.py`：单电机最小模板（默认 Damiao，其他厂商以注释模板给出）。
- `simple_02_quad_motor_control.py`：四电机最小多厂商往复示例（2 Damiao + 1 MyActuator + 1 RobStride）。

推荐顺序：

1. 先跑 `simple_01_motor_control.py`，验证 CAN 通道和单电机正常。
2. 再跑 `simple_02_quad_motor_control.py`，验证多控制器/多厂商并行控制。

快速运行命令：

```bash
# 1) 单电机（最稳妥起步）
PYTHONPATH=bindings/python/src LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH} \
python3 bindings/python/examples/simple_01_motor_control.py \
  --channel can0 --loop 120 --dt-ms 20 --pos 1.0 --vlim 1.0

# 2) 四电机往复
PYTHONPATH=bindings/python/src LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH} \
python3 bindings/python/examples/simple_02_quad_motor_control.py \
  --channel can0 --pos 1.0 --loop 240 --dt-ms 20 --swing-loop 60
```

### 参数速查（以上两个最简单脚本）

`simple_01_motor_control.py`
- `--channel`：CAN 接口名，如 `can0`、`can1`、`slcan0`。
- `--loop`：控制循环次数。
- `--dt-ms`：控制周期（毫秒）。建议先用 `20`；总线繁忙可调到 `30/50`。
- `--pos`：目标角度（弧度）。
- `--vlim`：POS_VEL 模式速度上限。

`simple_02_quad_motor_control.py`
- `--channel`：CAN 接口名。
- `--loop`：总循环次数。
- `--dt-ms`：控制周期（毫秒），同上建议。
- `--pos`：往复幅值（弧度），按 `+pos/-pos` 切换。
- `--swing-loop`：每 N 次循环切换一次方向。
- `--rs-dir-sign`：`-1` 表示 RobStride 方向与其余电机相反；`+1` 表示同向。
- `--dm-vlim`：Damiao 的 POS_VEL 速度上限。
- `--my-vlim`：MyActuator 的 POS_VEL 速度上限。
- `--rs-kp` / `--rs-kd`：RobStride 的 MIT 参数。

### 下一步看什么文档（推荐顺序）

1. [../README.zh-CN.md](../README.zh-CN.md) 或 [../README.md](../README.md)  
   看 Python binding 的安装与 API 总览。
2. [../../motor_cli/README.zh-CN.md](../../motor_cli/README.zh-CN.md) 或 [../../motor_cli/README.md](../../motor_cli/README.md)  
   用于扫描设备、确认 ID、查 CLI 参数。
3. [../../examples/README.zh-CN.md](../../examples/README.zh-CN.md) 或 [../../examples/README.md](../../examples/README.md)  
   查看更广泛的示例目录（web/WS/其他集成）。

### 新手必看关键点

- 一个 `Controller` 实例不能混用多厂商电机；多厂商请分多个 controller。
- `os error 105` 常见原因是发包过快，或有其它程序同时往 CAN 发包。
- 大多数控制链路需要周期发送，单次发送通常不足以稳定保持。
- 先扫描并确认 `motor-id/feedback-id`，再运行示例。

### WS 示例快速指南（`quad_vendor_binding_ws_demo.py`）

这是一个基于 Python binding 的四电机 WS 桥接后端（`dm1/dm2/my/rs`）+ 配套网页前端。

后端启动：

```bash
cd /path/to/rust_dm
python3 -m venv .venv-ws
source .venv-ws/bin/activate
pip install websockets
export PYTHONPATH=$PWD/bindings/python/src
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}
python3 bindings/python/examples/quad_vendor_binding_ws_demo.py \
  --bind 127.0.0.1 --port 9010 --channel can0 --dt-ms 20
```

前端启动（新终端）：

```bash
cd /path/to/rust_dm/bindings/python/examples
python3 -m http.server 8080
```

浏览器打开：
- `http://127.0.0.1:8080/quad_vendor_binding_ws_demo.html`
- WS 地址填：`ws://127.0.0.1:9010`

如果你的电机 ID 不同，启动后端时可覆盖：

```bash
python3 bindings/python/examples/quad_vendor_binding_ws_demo.py \
  --channel can0 \
  --dm1-id 0x01 --dm1-fid 0x11 --dm1-model 4340P \
  --dm2-id 0x07 --dm2-fid 0x17 --dm2-model 4310 \
  --my-id 1 --my-fid 0x241 --my-model X8 \
  --rs-id 127 --rs-fid 0xFE --rs-model rs-00
```

## 文件列表

- `simple_01_motor_control.py`：最简单单电机模板（默认 Damiao，含注释化厂商切换模板）
- `simple_02_quad_motor_control.py`：最简单四电机多厂商示例（固定周期 + 往复目标）
- `python_wrapper_demo.py`：最小 Damiao MIT 循环
- `damiao_maintenance_demo.py`：Damiao 维护流程（`clear_error` / `set_zero_position` / `set_can_timeout_ms` / `request_feedback`）
- `damiao_register_rw_demo.py`：Damiao 寄存器读写（`f32` + `u32` + 可选 `store_parameters`）
- `damiao_dm_serial_demo.py`：Damiao 串口桥传输示例（`Controller.from_dm_serial`）
- `dm_serial_01_calibration_demo.py`：SOP-01 串口桥维护/校准流程
- `dm_serial_02_control_modes_demo.py`：SOP-02 串口桥常规控制（4 模式，不做校准/配置写入）
- `dm_serial_03_status_demo.py`：SOP-03 串口桥状态监视
- `dm_serial_04_enable_setzero_no_delay_demo.py`：SOP-04 串口桥复现（`set_zero` 后立刻控制）
- `dm_serial_05_setzero_timing_ab_test.py`：SOP-05 串口桥置零等待 A/B 测试
- `dm_serial_06_recover_no_reboot_demo.py`：SOP-06 串口桥软件恢复（不重启主机）
- `dm_serial_07_enable_setzero_enable_rotate_demo.py`：SOP-07 串口桥稳健顺序（`disable -> set_zero -> enable -> control`）
- `dm_serial_08_negative_enable_setzero_guard_demo.py`：SOP-08 串口桥负向测试（`enable` 态 `set_zero` 应被核心拒绝）
- `dm_serial_leader_monitor_demo.py`：Damiao 串口桥 leader 监视（enable-all + 指定 ID 全状态流）
- `robstride_wrapper_demo.py`：RobStride ping / 读参 / mit / vel 示例
- `quad_vendor_pos_binding_demo.py`：Python binding 四电机同步位置示例（不经过 ws_gateway）
- `quad_vendor_binding_ws_demo.py`：Python binding WS 桥后端（用于网页控制）
- `quad_vendor_binding_ws_demo.html`：`quad_vendor_binding_ws_demo.py` 的网页 UI
- `hexfellow_canfd_demo.py`：Hexfellow CAN-FD 示例（仅 `mit` / `pos-vel`）
- `full_modes_demo.py`：Damiao 全模式示例
- `pid_register_tune_demo.py`：Damiao 参数调优示例
- `scan_ids_demo.py`：Damiao 快速扫描（历史辅助脚本）
- `pos_ctrl_demo.py`：Damiao 一次性位置指令
- `multi_motor_ctrl_demo.py`：Damiao 多电机控制（`-id` / `-pos` 一一映射）
- `mit_pos_switch_demo.py`：两阶段模式切换示例（MIT 后切 POS_VEL）
- `pos_repl_demo.py`：Damiao 交互式位置控制

## 快速运行（按厂商/场景）

Damiao：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/python_wrapper_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 20 --dt-ms 20
```

Damiao maintenance：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/damiao_maintenance_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --can-timeout-ms 1000 --set-zero 0
```

Damiao register rw：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/damiao_register_rw_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --read-f32-rid 21 --read-u32-rid 10 --store 0
```

Damiao dm-serial：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/damiao_dm_serial_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x01 --feedback-id 0x11 --mode mit --loop 40 --dt-ms 20
```

SOP-01 dm-serial calibration：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_01_calibration_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --set-zero 0 --can-timeout-ms 1000
```

SOP-02 dm-serial normal control（no calibration/config writes）：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_02_control_modes_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --mode pos-vel --pos 0.8 --vlim 1.0 --loop 100 --dt-ms 20
```

SOP-03 dm-serial status monitor：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_03_status_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --loop 100 --dt-ms 50
```

SOP-05 dm-serial set-zero settle-time A/B test：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_05_setzero_timing_ab_test.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --settle-list-ms 0,50,100,200 --rounds 10 --ensure-timeout-ms 500
```

SOP-06 dm-serial software recovery without reboot：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_06_recover_no_reboot_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --attempts 6 --timeout-ms 800
```

SOP-07 dm-serial robust set-zero + control sequence：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_07_enable_setzero_enable_rotate_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --target-pos 3.0 --vlim 1.0 \
  --loop 50 --dt-ms 20 --ensure-timeout-ms 800 \
  --post-setzero-ms 0
```

SOP-08 dm-serial negative test（`enable` then `set_zero`，expect guard reject）：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_08_negative_enable_setzero_guard_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  --motor-id 0x04 --feedback-id 0x14 --ensure-timeout-ms 800
```

## dm-serial 时序说明（置零序列）

- 现象：执行 `set_zero_position()` 后紧接着 `ensure_mode(...)`，可能触发 `register 10 not received` 超时。
- 根因模式：通常是 dm-serial 的时序窗口（桥接延迟 + 电机内部状态切换），不是普通控制模式逻辑错误。
- 核心守卫规则：`set_zero_position()` 仅在 `disable()` 之后允许。
- 核心稳定等待规则：`set_zero_position()` 后核心层固定等待 `20 ms`（Python 不暴露该参数）。
- 推荐稳健序列：
  1. `disable`（或 `disable_all`）
  2. `set_zero_position`
  3. `enable`（或 `enable_all`）
  4. `ensure_mode`
  5. 进入控制循环
- 一旦触发超时，先尝试软件恢复（`disable -> clear_error -> enable -> retry`），再考虑重启主机/设备。

Damiao dm-serial leader monitor（指定 ID 全状态流）：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/dm_serial_leader_monitor_demo.py \
  --serial-port /dev/ttyACM0 --serial-baud 921600 --model 4310 \
  -id 0x04 0x07 --loop 10000 --dt-ms 20 --hold-mode mit-zero
```

RobStride：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride 读参数：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/robstride_wrapper_demo.py \
  --channel can0 --model rs-00 --motor-id 127 --mode read-param --param-id 0x7019
```

Hexfellow（仅 CAN-FD）：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/hexfellow_canfd_demo.py \
  --channel can0 --motor-id 0x01 --feedback-id 0x00 --mode mit --loop 20 --dt-ms 50
```

CLI 统一扫描所有厂商：

```bash
PYTHONPATH=bindings/python/src python3 -m motorbridge.cli scan \
  --vendor all --channel can0 --start-id 0x01 --end-id 0xFF
```

多电机 `pos-vel`（示例：电机 `4` 和 `7`，位置按顺序映射）：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/multi_motor_ctrl_demo.py \
  --channel can0 --model 4310 --mode pos-vel \
  -id 4 7 -pos 0.8 -0.6 -vlim 1.2 1.2 --loop 200 --dt-ms 20
```

MIT/POS_VEL 切换示例（仅 POS_VEL 运行，电机 `4` 和 `7`，目标 `-3`）：

```bash
PYTHONPATH=bindings/python/src python3 bindings/python/examples/mit_pos_switch_demo.py \
  --channel can0 --model 4310 -id 4 7 \
  --trajectory -3 \
  --mit-hold-loops 0 --pos-hold-loops 50 \
  --dt-ms 20 --print-state 1
```

## Damiao 覆盖说明

Damiao 示例已经覆盖高层 SDK 的完整常用面：

- 控制模式：`mit` / `pos-vel` / `vel` / `force-pos`
- 传输路径：`Controller(channel)` + `Controller.from_dm_serial(...)`
- 维护接口：`clear_error`、`set_zero_position`、`set_can_timeout_ms`、`request_feedback`
- 寄存器接口：`get/write f32`、`get/write u32`、`store_parameters`
- 扫描与调参流程

## 变更范围说明

- 最近多电机示例更新主要集中在 `examples/*` 和 WS bridge 脚本。
- 本批次未修改 `motor_core` 行为；WS 相关调优在 `integrations/ws_gateway`。
