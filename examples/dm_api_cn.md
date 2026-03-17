# Damiao API 与调参总表（完整版）

本页是 `motorbridge` 当前 Damiao 可调控制/配置参数的实用总表。

> English version: [dm_api.md](dm_api.md)

## 1）通用设备参数

| 参数 | 含义 | 常用值 |
|---|---|---|
| `channel` | CAN 接口名 | `can0` |
| `model` | 电机型号字符串（`4310`、`4340P` 等） | 与实物一致 |
| `motor-id` | 命令 ID（`ESC_ID`） | 如 `0x01` |
| `feedback-id` | 反馈 ID（`MST_ID`） | 如 `0x11` |
| `loop` | 发送循环次数 | `100` |
| `dt-ms` | 每次发送间隔（毫秒） | `20` |

## 2）实时控制模式参数

## 2.1 MIT 模式

参数：

- `pos`：目标位置
- `vel`：目标速度
- `kp`：位置刚度增益
- `kd`：速度阻尼增益
- `tau`：前馈力矩

示例（`motor_cli`）：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 200 --dt-ms 20
```

## 2.2 POS_VEL 模式

参数：

- `pos`：目标位置
- `vlim`：速度限制

示例：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 300 --dt-ms 20
```

## 2.3 VEL 模式

参数：

- `vel`：目标速度

示例：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 --loop 100 --dt-ms 20
```

## 2.4 FORCE_POS 模式

参数：

- `pos`：目标位置
- `vlim`：速度限制
- `ratio`：力矩限制比例

示例：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 --loop 100 --dt-ms 20
```

## 2.5 模式寄存器（`CTRL_MODE`）

寄存器：

- `rid=10`（`CTRL_MODE`），取值：
  - `1=MIT`
  - `2=POS_VEL`
  - `3=VEL`
  - `4=FORCE_POS`

## 3）强影响寄存器（优先）

这些参数对控制效果影响很大，调参要谨慎。

| RID | 名称 | 类型 | 含义 |
|---|---|---|---|
| `21` | `PMAX` | `f32` | 位置映射范围 |
| `22` | `VMAX` | `f32` | 速度映射范围 |
| `23` | `TMAX` | `f32` | 力矩映射范围 |
| `25` | `KP_ASR` | `f32` | 速度环 Kp |
| `26` | `KI_ASR` | `f32` | 速度环 Ki |
| `27` | `KP_APR` | `f32` | 位置环 Kp |
| `28` | `KI_APR` | `f32` | 位置环 Ki |
| `4` | `ACC` | `f32` | 加速度 |
| `5` | `DEC` | `f32` | 减速度 |
| `6` | `MAX_SPD` | `f32` | 最大速度 |
| `9` | `TIMEOUT` | `u32` | 通信超时寄存器 |

## 4）保护相关寄存器

| RID | 名称 | 类型 | 含义 |
|---|---|---|---|
| `0` | `UV_Value` | `f32` | 欠压阈值 |
| `2` | `OT_Value` | `f32` | 过温阈值 |
| `3` | `OC_Value` | `f32` | 过流阈值 |
| `29` | `OV_Value` | `f32` | 过压阈值 |

## 5）参数读写方法

推荐流程：

1. `get_register` 读取旧值
2. `write_register` 写入新值
3. 再次 `get_register` 回读确认
4. `store_parameters` 持久化

Python SDK API 示例：

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_damiao_motor(0x01, 0x11, "4340P")
    old = m.get_register_f32(22, 1000)
    print("old VMAX", old)
    m.write_register_f32(22, old * 0.9)
    new = m.get_register_f32(22, 1000)
    print("new VMAX", new)
    m.store_parameters()
    m.close()
```

WS 网关命令示例：

```json
{"op":"get_register_f32","rid":22,"timeout_ms":1000}
{"op":"write_register_f32","rid":22,"value":25.0}
{"op":"store_parameters"}
```

## 6）ID 与标定参数

- `rid=8`（`ESC_ID`）：命令 ID
- `rid=7`（`MST_ID`）：反馈 ID

工具入口：

- Rust：`tools/motor_calib`（`scan`、`set-id`、`verify`）
- Python：`motorbridge-cli scan/id-set/id-dump`
- WS：`scan`、`set_id`、`verify`

## 7）安全建议

- 每次只调一组参数。
- 小步进调整，每一步都回读确认。
- 保持机械安全和急停预案。
- 对保护阈值（`0/2/3/29`）不要盲目激进调整。
