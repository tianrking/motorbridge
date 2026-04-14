# RobStride 三通道对照手册（Core CLI / Python CLI / Python SDK）

本文只聚焦 RobStride，并按“每个功能点三种方式”组织：
- Core CLI：`motor_cli`（Rust，基准实现）
- Python CLI：`motorbridge-cli` 或 `python -m motorbridge.cli`
- Python 代码：`from motorbridge import Controller, Mode`

## 通道说明（仅 SocketCAN）

- RobStride 在本手册中只讨论 SocketCAN 路径（`can0`、`can1`）。
- 不涉及其他传输路径。
- Linux 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 无效）。
- 详细排障可参考：`../../docs/zh/can_debugging.md`。

## 0）前置

### 0.1 环境

在仓库根目录：

```bash
cd motorbridge
```

Core CLI（建议先编译一次）：

```bash
cargo build -p motor_cli --release
CLI=./target/release/motor_cli
```

Python CLI（源码联调建议带动态库路径）：

```bash
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}
```

### 0.2 通用参数（示例）

```bash
CH=can0
MODEL=rs-06
MID=127
FID=0xFD
```

## 1）扫描（scan）

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --mode scan --start-id 120 --end-id 130
```

### Python CLI

```bash
motorbridge-cli scan \
  --vendor robstride --channel "$CH" --model "$MODEL" \
  --start-id 120 --end-id 130 --feedback-ids 0xFD --param-timeout-ms 60
```

### Python 代码

```python
from motorbridge import Controller

found = []
with Controller("can0") as ctrl:
    for mid in range(120, 131):
        try:
            m = ctrl.add_robstride_motor(mid, 0xFD, "rs-06")
            try:
                print(mid, m.robstride_ping())
                found.append(mid)
            finally:
                m.close()
        except Exception:
            pass
print("found:", found)
```

## 2）连通性（ping）

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" --mode ping
```

### Python CLI

```bash
motorbridge-cli run \
  --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode ping
```

### Python 代码

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    print(m.robstride_ping())
    m.close()
```

## 3）使能（enable）

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" --mode enable
```

### Python CLI

```bash
motorbridge-cli run \
  --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode enable --loop 1 --dt-ms 20
```

### Python 代码

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    m.enable()
    m.close()
```

## 4）失能（disable）

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" --mode disable
```

### Python CLI

```bash
motorbridge-cli run \
  --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode disable --loop 1 --dt-ms 20
```

### Python 代码

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    m.disable()
    m.close()
```

## 4.5）模式速查：统一模式 vs 原生模式（RobStride）

统一模式（推荐业务层）：
- `MIT`：统一 5 参数 `pos/vel/kp/kd/tau`，映射到 RobStride 原生 MIT 控制帧。
- `POS_VEL`：统一位置模式，映射到原生 Position：
  - `run_mode=1`
  - `vlim -> 0x7017 (limit_spd)`
  - `pos -> 0x7016 (loc_ref)`
- `VEL`：统一速度模式，映射到原生 Velocity：
  - `run_mode=2`
  - `vel -> 0x700A (spd_ref)`

原生模式（协议级/调试）：
- 通过 `read-param / write-param` 直接读写参数。
- 常见参数：
  - `0x7005`：`run_mode`
  - `0x7016`：`loc_ref`
  - `0x7017`：`limit_spd`
  - `0x700A`：`spd_ref`
  - `0x7019`：`mechPos`
  - `0x701B`：`mechVel`

参数有效性：
- `MIT`：`pos/vel/kp/kd/tau` 全有效。
- `POS_VEL`：仅 `pos/vlim/(可选 kp/loc-kp)` 有效；
  `vel/kd/tau` 在该模式下无效（会被忽略）。

## 5）MIT

参数映射与有效性（RobStride）：
- 有效参数：`pos`、`vel`、`kp`、`kd`、`tau`（五个都有效）。
- 单位语义：`pos(rad)`、`vel(rad/s)`、`tau(Nm)`；`kp/kd` 为 MIT 闭环增益。
- 这是 RobStride 对齐统一协议后最完整的控制模式。

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode mit --pos 0 --vel 0 --kp 3 --kd 0.2 --tau 0 --loop 40 --dt-ms 50
```

### Python CLI

```bash
motorbridge-cli run \
  --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode mit --pos 0 --vel 0 --kp 3 --kd 0.2 --tau 0 --loop 40 --dt-ms 50
```

### Python 代码

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    m.enable()
    m.ensure_mode(Mode.MIT, 1000)
    for _ in range(40):
        m.send_mit(0.0, 0.0, 3.0, 0.2, 0.0)
    m.close()
```

## 6）POS_VEL

参数映射与有效性（RobStride）：
- 有效参数：`pos`、`vlim`，以及可选 `kp/loc-kp`。
- 映射关系：`run_mode=1(Position)`，`vlim -> 0x7017(limit_spd)`，`pos -> 0x7016(loc_ref)`。
- 无效参数：`vel`、`kd`、`tau`（在该模式下会被忽略，不参与控制）。

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode pos-vel --pos 1.0 --vlim 0.8 --loop 1 --dt-ms 20
```

### Python CLI

```bash
motorbridge-cli run \
  --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode pos-vel --pos 1.0 --vlim 0.8 \
  --ensure-mode 1 --ensure-timeout-ms 1500 --ensure-strict 1 \
  --loop 1 --dt-ms 20
```

说明：
- `pos-vel` 建议 `loop=1` 作为“到位指令”。
- 若需要连续闭环调节，通常 MIT 更稳、更可控。

### Python 代码

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    m.enable()
    m.ensure_mode(Mode.POS_VEL, 1500)
    m.send_pos_vel(1.0, 0.8)
    m.close()
```

## 7）VEL

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

### Python CLI

```bash
motorbridge-cli run \
  --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

### Python 代码

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    m.enable()
    m.ensure_mode(Mode.VEL, 1000)
    for _ in range(40):
        m.send_vel(0.3)
    m.close()
```

## 8）读参数（read-param）

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode read-param --param-id 0x7019
```

### Python CLI

```bash
motorbridge-cli robstride-read-param \
  --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --param-id 0x7019 --type f32 --timeout-ms 200
```

### Python 代码

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    print(m.robstride_get_param_f32(0x7019, 200))
    m.close()
```

## 9）写参数（write-param）

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode write-param --param-id 0x700A --param-value 0.3
```

### Python CLI

```bash
motorbridge-cli robstride-write-param \
  --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --param-id 0x700A --type f32 --value 0.3 --verify 1 --timeout-ms 200
```

### Python 代码

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    m.robstride_write_param_f32(0x700A, 0.3)
    print(m.robstride_get_param_f32(0x700A, 200))
    m.close()
```

## 10）改 ID（set-motor-id）

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --set-motor-id 126 --store 1
```

### Python CLI

当前 `motorbridge-cli` 无 RobStride 专用改 ID 子命令。

### Python 代码

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    m.robstride_set_device_id(126)
    m.store_parameters()
    m.close()
```

## 11）写零点（zero）

### Core CLI

```bash
$CLI --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode zero --zero-exp 1 --store 1
```

### Python CLI

```bash
motorbridge-cli run \
  --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode zero --zero-exp 1 --store 1
```

等价别名：

```bash
motorbridge-cli run \
  --vendor robstride --channel "$CH" --model "$MODEL" --motor-id "$MID" --feedback-id "$FID" \
  --mode set-zero --zero-exp 1 --store 1
```

### Python 代码

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    m.disable()
    m.set_zero_position()
    m.store_parameters()
    m.close()
```

验证建议：

```bash
motorbridge-cli robstride-read-param \
  --channel "$CH" --model "$MODEL" --motor-id 127 --feedback-id "$FID" \
  --param-id 0x7019 --type f32 --timeout-ms 200
```

## 12）结论（准确性口径）

- 以 Core CLI（`motor_cli`）为基准最稳。
- Python CLI/SDK 与 core 大部分功能对齐；RobStride `zero/set-zero` 已支持（`run --mode zero|set-zero --zero-exp 1`），`set-id` 仍建议使用 Core CLI 或 SDK 代码。
- 需要严格复现时，请优先按本文 Core CLI 命令执行。
