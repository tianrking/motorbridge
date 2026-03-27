# motor_cli（中文）

Rust `motor_cli` 的全参数完整说明。

- Crate: `motor_cli`
- 推荐（release 压缩包）：`./bin/motor_cli [参数...]`
- 可选（源码编译后）：`./target/release/motor_cli [参数...]`

## 优先使用 Release 二进制

先从 GitHub Releases 下载并解压对应包（例如 `motor-cli-vX.Y.Z-linux-x86_64.tar.gz`），再直接运行：

```bash
./bin/motor_cli -h
./bin/motor_cli --vendor damiao --mode scan --start-id 1 --end-id 16
```

如果你希望直接输入 `motor_cli` 命令：

```bash
export PATH="$(pwd)/bin:$PATH"
motor_cli -h
```

## Damiao 指令与寄存器进阶文档

- 中文详表（指令/寄存器/调参）: `DAMIAO_API.zh-CN.md`
- English version: `DAMIAO_API.md`

## RobStride 指令与参数进阶文档

- 中文详表（参数/能力边界）: `ROBSTRIDE_API.zh-CN.md`
- English version: `ROBSTRIDE_API.md`

## MyActuator 指令与模式进阶文档

- 中文详表（命令/模式/参数）: `MYACTUATOR_API.zh-CN.md`
- English version: `MYACTUATOR_API.md`

## HighTorque 补充说明

- 协议深度分析文档：`../docs/zh/hightorque_protocol_analysis.md`
- 当前 `vendor=hightorque` 为 原生 ht_can v1.5.5 的“直连 CAN”模式，不是官方的“串口->CANboard”传输链路。

## CAN 调试入口

- Linux `slcan` + Windows `pcan` 专业排障：`../docs/zh/can_debugging.md`
- English guide: `../docs/en/can_debugging.md`

## 传输标识

- `[STD-CAN]` => `--transport auto|socketcan`
- `[CAN-FD]` => `--transport socketcanfd`（仅 Damiao，Linux）
- `[DM-SERIAL]` => `--transport dm-serial`（仅 Damiao）

当前状态：`[CAN-FD]` 仅表示链路已接入，电机验证矩阵待补。

## 1. 参数解析规则

- 仅解析 `--key value` 形式。
- 单独开关（如 `--help`）会按值 `1` 处理。
- ID 类参数支持十进制（如 `20`）与十六进制（如 `0x14`）。
- 未被代码使用的参数即使传入，也不会生效。

### 1.1 统一调用范式（CLI 大一统）

所有品牌都遵循同一个调用骨架，只是 `vendor/model/mode` 与附加参数不同：

```bash
motor_cli \
  --vendor <damiao|robstride|hightorque|myactuator|all> \
  --transport <auto|socketcan|socketcanfd|dm-serial> \
  --channel <can0|slcan0|can0@1000000...> \
  --model <model-name> \
  --motor-id <id> --feedback-id <id> \
  --mode <mode-name> \
  [模式参数...] \
  --loop <n> --dt-ms <ms>
```

说明：
- `socketcanfd` 与 `dm-serial` 仅 Damiao 可用；其他品牌统一走标准 CAN（SocketCAN/PCAN）。
- `vendor=all` 当前仅用于统一扫描（`--mode scan`）。

### 1.2 通用参数语义（先理解这些）

| 参数 | 语义 |
|---|---|
| `--vendor` | 选择品牌驱动实现（统一入口下发到不同 vendor backend） |
| `--transport` | 选择传输层（标准 CAN 或 Damiao 串口桥） |
| `--channel` | CAN 通道名（Linux 为网卡名；Windows 可带 `@bitrate`） |
| `--model` | 型号名称，用于该品牌下的限值/能力边界与编码映射 |
| `--motor-id` | 目标电机 ID（发送命令目标） |
| `--feedback-id` | 反馈帧 ID（接收状态来源） |
| `--mode` | 控制/查询动作类型（不同品牌支持集合不同） |
| `--loop` / `--dt-ms` | 循环发送次数 / 周期 |
| `--ensure-mode` | 控制前是否自动切控制模式（Damiao 等支持） |

### 1.3 各品牌参数变量怎么传（统一调用下的差异）

| 品牌 | `--model` 传入 | `--motor-id` / `--feedback-id` 传入 | 常用 `--mode` |
|---|---|---|---|
| Damiao | 必传且建议按电机真实型号（混型场景不要写死一个 model） | `motor-id` 与 `feedback-id` 都需要按实际设备传入 | `scan`、`enable`、`disable`、`mit`（当前串口桥建议这四个） |
| RobStride | 传 `rs-00/01...` 等 | `motor-id` 必传；`feedback-id` 常用 `0xFF` | `ping`、`scan`、`mit`、`vel`、`read-param`、`write-param` |
| HighTorque | 传 `hightorque`（hint） | 按设备 ID 传入 | `read`、`mit`、`pos`、`vel`、`tqe`、`scan` 等 |
| MyActuator | 传运行时型号字符串（默认 `X8`） | 标准 11-bit 规则（常用 `0x140+id` / `0x240+id`） | `status`、`scan`、`current`、`vel`、`pos`、`enable/disable` |
| all | 分品牌 hint（`--damiao-model` 等） | 仅扫描场景使用 | `scan` |

## 2. 顶层通用参数（所有 vendor）

| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `--help` | flag | 关闭 | 输出帮助并退出 |
| `--vendor` | string | `damiao` | `damiao` / `robstride` / `hightorque` / `myactuator` / `hexfellow` / `all` |
| `--transport` | string | `auto` | `auto` / `socketcan` / `socketcanfd` / `dm-serial`（`socketcanfd` 与 `dm-serial` 仅 Damiao） |
| `--channel` | string | `can0` | Linux：SocketCAN 网卡名（`can0`/`slcan0`）；Windows（PCAN 后端）：`can0`/`can1`，可加 `@bitrate`（如 `can0@1000000`） |
| `--serial-port` | string | `/dev/ttyACM0` | `--transport dm-serial` 时使用 |
| `--serial-baud` | u64 | `921600` | `--transport dm-serial` 时使用 |
| `--model` | string | 按 vendor 决定 | Damiao 默认 `4340`；RobStride 默认 `rs-00`；HighTorque 默认 `hightorque`；MyActuator 默认 `X8` |
| `--motor-id` | u16(hex/dec) | `0x01` | 电机 CAN ID |
| `--feedback-id` | u16(hex/dec) | 按 vendor 决定 | Damiao 默认 `0x11`；RobStride 默认 `0xFF`；HighTorque 默认 `0x01`；MyActuator 默认 `0x241`（motor-id=1） |
| `--mode` | string | 按 vendor 决定 | Damiao 默认 `mit`；RobStride 默认 `ping`；HighTorque 默认 `read`；MyActuator 默认 `status`；`all` 默认 `scan` |
| `--loop` | u64 | `1` | 控制循环次数 |
| `--dt-ms` | u64 | `20` | 循环间隔毫秒 |
| `--ensure-mode` | `0/1` | `1` | 控制前自动切模式 |

### 2.1 通道速查（`--channel`）

- Linux SocketCAN：
  - 直接使用网卡名：`can0`、`can1`、`slcan0`。
  - 波特率在网卡初始化阶段设置（`ip link` / `slcand`），不要写到 `--channel`。
  - `can0@1000000` 在 Linux SocketCAN 下无效。
- Windows PCAN：
  - `can0` 映射 `PCAN_USBBUS1`，`can1` 映射 `PCAN_USBBUS2`。
  - 支持可选波特率后缀：`can0@1000000`。

### 2.2 Damiao 串口桥速查（`--transport dm-serial`）

- 该链路为适配器私有路径，面向 Damiao 电机。
- 常用参数：`--transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600`。
- `dm-serial` 模式下，传输层创建会忽略 `--channel`。
- `dm-serial` 仅改变“传输层”（走串口桥），Damiao 的业务参数与模式接口保持一致（`--mode`、`--motor-id`、`--feedback-id`、`--verify-model`、`--ensure-mode` 等）。

### 2.3 Damiao 独立 CAN-FD 链路速查（`--transport socketcanfd`）

- 该链路为 Linux 专用，并与经典 `socketcan` 链路并存。
- 常用参数：`--transport socketcanfd --channel can0`。
- 使用前先确保网口处于 FD 模式（`scripts/canfd_restart.sh can0`）。
- 当前状态：链路已接入，尚未标注“已完成 CAN-FD 电机验证”的型号列表。

## 3. vendor=`damiao`

### 3.1 支持模式

- `scan`
- `enable`
- `disable`
- `mit`
- `pos-vel`
- `vel`
- `force-pos`

### 3.2 Damiao 专用参数

| 参数 | 类型 | 默认值 | 作用范围 | 说明 |
|---|---|---|---|---|
| `--verify-model` | `0/1` | `1` | 非 scan | 校验 PMAX/VMAX/TMAX 与 `--model` 一致 |
| `--verify-timeout-ms` | u64 | `500` | 非 scan | 型号握手读取超时 |
| `--verify-tol` | f32 | `0.2` | 非 scan | 限值匹配容差 |
| `--start-id` | u16 | `1` | scan | 扫描起始 ID（1..255） |
| `--end-id` | u16 | `255` | scan | 扫描结束 ID（1..255） |
| `--set-motor-id` | u16 可选 | 无 | 改 ID 流程 | 写 ESC_ID（RID 8） |
| `--set-feedback-id` | u16 可选 | 无 | 改 ID 流程 | 写 MST_ID（RID 7） |
| `--store` | `0/1` | `1` | 改 ID 流程 | 是否保存参数 |
| `--verify-id` | `0/1` | `1` | 改 ID 流程 | 是否回读 RID7/RID8 校验 |

### 3.3 各模式控制参数

| 模式 | 参数 | 默认值 |
|---|---|---|
| `mit` | `--pos --vel --kp --kd --tau` | `0 0 2 1 0` |
| `pos-vel` | `--pos --vlim` | `0 1.0` |
| `vel` | `--vel` | `0` |
| `force-pos` | `--pos --vlim --ratio` | `0 1.0 0.1` |
| `enable` / `disable` | 无额外参数 | n/a |

### 3.4 扫描行为细节

- 扫描逻辑本质上是“型号无关”的：内部会遍历内置 model-hint 列表。
- 每个候选 ID 会尝试多个 feedback-hint：推断值（`id+0x10`）、用户给定 `--feedback-id`、`0x11`、`0x17`。
- 优先用寄存器（RID 21/22/23）检测，失败再走反馈回退检测。

### 3.5 Damiao 示例

```bash
# 扫描 1..16
motor_cli \
  --vendor damiao --channel can0 --mode scan --start-id 1 --end-id 16
# [STD-CAN]

# MIT 控制
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --pos 1.57 --vel 2.0 --kp 35 --kd 1.2 --tau 0.3 --loop 120 --dt-ms 20
# [STD-CAN]

# 通过 Damiao 串口桥执行 MIT
motor_cli \
  --vendor damiao --transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600 \
  --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --verify-model 0 --ensure-mode 0 \
  --pos 1.0 --vel 0 --kp 2 --kd 1 --tau 0 --loop 80 --dt-ms 20
# [DM-SERIAL]

# 位置速度控制
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode pos-vel --pos 3.14 --vlim 4.0 --loop 120 --dt-ms 20
# [STD-CAN]

# 改 ID + 保存 + 校验
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --set-motor-id 0x04 --set-feedback-id 0x14 --store 1 --verify-id 1
```

### 3.6 Damiao 串口桥完整接口与用法（`--transport dm-serial`）

先定义公共前缀（建议）：

```bash
DM_SERIAL="--vendor damiao --transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600 --model 4310"
```

#### 3.6.1 串口桥下必用/常用参数

| 参数 | 是否建议显式传入 | 说明 |
|---|---|---|
| `--transport dm-serial` | 必须 | 切到 Damiao 串口桥链路 |
| `--serial-port` | 必须 | 串口设备，如 `/dev/ttyACM1` |
| `--serial-baud` | 必须 | 串口波特率，常用 `921600` |
| `--channel` | 可省略 | 该模式下会被忽略 |
| `--motor-id` / `--feedback-id` | 控制时必须 | 与扫描命中结果一致 |
| `--verify-model` | 建议按现场开关 | 若握手链路不稳定可先设 `0` 做联通验证 |
| `--ensure-mode` | 建议按现场开关 | 若电机模式切换流程不稳定可先设 `0` |

> 当前串口桥场景对外推荐仅使用：`scan` / `enable` / `disable` / `mit`。

#### 3.6.2 串口桥下常用四模式命令模板

```bash
# 1) 扫描
motor_cli $DM_SERIAL --mode scan --start-id 1 --end-id 16

# 2) 使能
motor_cli $DM_SERIAL --motor-id 0x04 --feedback-id 0x14 --mode enable --verify-model 0 --loop 1

# 3) 失能
motor_cli $DM_SERIAL --motor-id 0x04 --feedback-id 0x14 --mode disable --verify-model 0 --loop 1

# 4) MIT
motor_cli $DM_SERIAL --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --verify-model 0 --ensure-mode 0 \
  --pos 0.5 --vel 0 --kp 2 --kd 1 --tau 0 --loop 80 --dt-ms 20
```

#### 3.6.3 推荐测试顺序

1. `scan` 先确认在线 ID。
2. `enable --loop 1` 做最小动作验证。
3. `mit` 小步参数（小 `pos`、中低 `kp/kd`）验证控制闭环。
4. 最后再上业务参数与连续循环。

## 4. vendor=`robstride`

### 4.1 支持模式

- `ping`
- `scan`
- `enable`
- `disable`
- `mit`
- `vel`
- `read-param`
- `write-param`

### 4.2 RobStride 专用参数

| 参数 | 类型 | 默认值 | 作用范围 | 说明 |
|---|---|---|---|---|
| `--start-id` | u16 | `1` | scan | 扫描起始 ID（1..255） |
| `--end-id` | u16 | `255` | scan | 扫描结束 ID（1..255） |
| `--manual-vel` | f32 | `0.2` | scan 回退 | 盲探速度 |
| `--manual-ms` | u64 | `200` | scan 回退 | 每个 ID 脉冲时长 |
| `--manual-gap-ms` | u64 | `200` | scan 回退 | ID 间隔 |
| `--set-motor-id` | u16 可选 | 无 | 改 ID 流程 | 设置新设备 ID |
| `--store` | `0/1` | `1` | 改 ID 流程 | 保存参数 |
| `--param-id` | u16 | 参数模式必填 | 读写参数 | 参数 ID |
| `--param-value` | 类型化值 | 写参数必填 | write-param | 按参数元数据解析 |

### 4.3 各模式控制参数

| 模式 | 参数 | 默认值 |
|---|---|---|
| `mit` | `--pos --vel --kp --kd --tau` | `0 0 8 0.2 0` |
| `vel` | `--vel` | `0` |
| `enable` / `disable` | 无额外参数 | n/a |

### 4.4 扫描行为细节

- 第一阶段：每个 ID 做 `ping` + 参数查询探测。
- 全范围无命中时：进入盲探速度脉冲模式（人工观察是否转动）。
- 回退阶段若有状态反馈，也会计入命中。

### 4.5 RobStride 示例

```bash
# ping
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFF --mode ping

# 扫描
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 255

# MIT 控制
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFF \
  --mode mit --pos 3.14 --vel 0 --kp 8 --kd 0.4 --tau 1.5 --loop 120 --dt-ms 20

# 速度模式
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFF \
  --mode vel --vel 2.0 --loop 100 --dt-ms 20

# 读参数
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFF \
  --mode read-param --param-id 0x7005

# 写参数
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFF \
  --mode write-param --param-id 0x7005 --param-value 2
```

## 5. vendor=`all`

`vendor=all` 当前仅支持 `--mode scan`。

### 5.1 all-scan 额外参数

| 参数 | 默认值 | 说明 |
|---|---|---|
| `--damiao-model` | `4340P` | 传给 Damiao 扫描流程的 model hint |
| `--robstride-model` | `rs-00` | 传给 RobStride 扫描流程的 model hint |
| `--hightorque-model` | `hightorque` | 传给 HighTorque 扫描流程的 model hint |
| `--myactuator-model` | `X8` | 传给 MyActuator 扫描流程的 model hint |
| `--start-id` | `1` | 同时传给各扫描流程 |
| `--end-id` | `255` | 传给 Damiao/RobStride；MyActuator 会自动截断到 `32` |

### 5.2 示例

```bash
motor_cli \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```

## 5.3 vendor=`hightorque`（原生 `ht_can` v1.5.5）

- 当前实现走 HighTorque 原生 `ht_can` v1.5.5 直连 CAN 协议路径。
- 用于 SocketCAN（`can0` 等）直连电机场景。
- HighTorque 官方 Panthera SDK 的“USB 串口 -> CANboard -> 电机”链路与当前 CLI 直连 CAN 路径相互独立。
- 支持模式：`scan | read | ping | mit | pos | vel | tqe | pos-vel-tqe | volt | cur | stop | brake | rezero | conf-write | timed-read`。
- 统一单位接口：
  - `--pos` 为 `rad`
  - `--vel` 为 `rad/s`
  - `--tau` 为 `Nm`
  - `--kp`、`--kd` 为统一 MIT 参数签名保留，`ht_can` 协议本身不使用。
  - 原始调试参数：`--raw-pos`、`--raw-vel`、`--raw-tqe`。

## 6. vendor=`myactuator`

### 6.1 支持模式

- `scan`
- `enable`
- `disable`
- `stop`
- `set-zero`
- `status`
- `current`
- `vel`
- `pos`
- `version`
- `mode-query`

### 6.2 MyActuator 专用参数

| 参数 | 类型 | 默认值 | 作用范围 | 说明 |
|---|---|---|---|---|
| `--start-id` | u16 | `1` | scan | 扫描起始 ID（1..32） |
| `--end-id` | u16 | `32` | scan | 扫描结束 ID（1..32，传入大于 32 会自动截断） |
| `--current` | f32 | `0.0` | current | 电流目标值（A） |
| `--vel` | f32 | `0.0` | vel | 速度目标值（rad/s，内部转换为 deg/s） |
| `--pos` | f32 | `0.0` | pos | 绝对位置目标值（rad，内部转换为 deg） |
| `--max-speed` | f32 | `8.726646` | pos | 位置模式最大速度（rad/s，内部转换） |

状态输出说明：

- `angle` 来自 `0x9C` 状态2近圈角。
- `mt_angle` 来自 `0x92` 多圈角，绝对位置判定应优先看它。

### 6.3 MyActuator 示例

```bash
# 扫描 1..32
motor_cli \
  --vendor myactuator --channel can0 --mode scan --start-id 1 --end-id 32

# 连续状态读取
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode status --loop 40 --dt-ms 50

# 速度模式
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode vel --vel 0.5236 --loop 100 --dt-ms 20

# 位置模式
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode pos --pos 3.1416 --max-speed 5.236 --loop 1

# 将当前位置设为零点（持久生效需断电重启）
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode set-zero --loop 1
```

## 7. vendor=`hexfellow`

链路限制：
- Hexfellow 在本仓库按“仅 CAN-FD”接入（`--transport socketcanfd`）。
- 当前支持范围：`scan / status / pos-vel / mit / enable / disable`。
- 当前状态：链路已接入，电机验证矩阵待补。

### 7.1 Hexfellow 示例

```bash
# 扫描 ID
motor_cli \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --mode scan --start-id 1 --end-id 32

# 状态查询
motor_cli \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --model hexfellow --motor-id 1 --feedback-id 0 \
  --mode status

# 位置速度（pos 单位 rad，vlim 单位 rad/s）
motor_cli \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --model hexfellow --motor-id 1 --feedback-id 0 \
  --mode pos-vel --pos 3.1415926 --vlim 2.0

# MIT（pos/vel 单位 rad/rad/s）
motor_cli \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --model hexfellow --motor-id 1 --feedback-id 0 \
  --mode mit --pos 0.0 --vel 0.0 --kp 1000 --kd 100 --tau 0
```

## 8. 实用建议

- Damiao 改 ID 建议始终使用 `--store 1 --verify-id 1`。
- 若扫描偶发漏检，重启 CAN 后重试。
- RobStride 没有 CLI 的 `send_pos_vel` 模式，请用 `mit` 或 `vel`。
- MyActuator 若 `0x9A` 返回错误码 `0x0004`（欠压），电机会在线但不转，需要先恢复供电电压。
