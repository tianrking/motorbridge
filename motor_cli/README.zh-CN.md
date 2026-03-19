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

## 1. 参数解析规则

- 仅解析 `--key value` 形式。
- 单独开关（如 `--help`）会按值 `1` 处理。
- ID 类参数支持十进制（如 `20`）与十六进制（如 `0x14`）。
- 未被代码使用的参数即使传入，也不会生效。

## 2. 顶层通用参数（所有 vendor）

| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `--help` | flag | 关闭 | 输出帮助并退出 |
| `--vendor` | string | `damiao` | `damiao` / `robstride` / `all` |
| `--channel` | string | `can0` | SocketCAN 通道 |
| `--model` | string | 按 vendor 决定 | Damiao 默认 `4340`；RobStride 默认 `rs-00` |
| `--motor-id` | u16(hex/dec) | `0x01` | 电机 CAN ID |
| `--feedback-id` | u16(hex/dec) | 按 vendor 决定 | Damiao 默认 `0x11`；RobStride 默认 `0xFF` |
| `--mode` | string | 按 vendor 决定 | Damiao 默认 `mit`；RobStride 默认 `ping`；`all` 默认 `scan` |
| `--loop` | u64 | `1` | 控制循环次数 |
| `--dt-ms` | u64 | `20` | 循环间隔毫秒 |
| `--ensure-mode` | `0/1` | `1` | 控制前自动切模式 |

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
| `mit` | `--pos --vel --kp --kd --tau` | `0 0 30 1 0` |
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

# MIT 控制
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --pos 1.57 --vel 2.0 --kp 35 --kd 1.2 --tau 0.3 --loop 120 --dt-ms 20

# 位置速度控制
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode pos-vel --pos 3.14 --vlim 4.0 --loop 120 --dt-ms 20

# 改 ID + 保存 + 校验
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --set-motor-id 0x04 --set-feedback-id 0x14 --store 1 --verify-id 1
```

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
| `--start-id` | `1` | 同时传给两类扫描 |
| `--end-id` | `255` | 同时传给两类扫描 |

### 5.2 示例

```bash
motor_cli \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```

## 6. 实用建议

- Damiao 改 ID 建议始终使用 `--store 1 --verify-id 1`。
- 若扫描偶发漏检，重启 CAN 后重试。
- RobStride 没有 CLI 的 `send_pos_vel` 模式，请用 `mit` 或 `vel`。
