# RobStride API 能力对照与完善空间（详版）

> 评估日期：2026-03-19  
> 目标：对比 `motorbridge` 当前 RobStride 能力 vs `Python_Sample`、`robstride_actuator_bridge`、以及 RobStride 协议常量层，找出可补齐空间。

## 1. 评估基线（本次对照来源）

- `rust_dm`
  - `motor_vendors/robstride/src/protocol.rs`
  - `motor_vendors/robstride/src/motor.rs`
  - `motor_cli/src/main.rs`
  - `motor_abi/src/lib.rs`
  - `bindings/python/src/motorbridge/core.py`
  - `integrations/ws_gateway/src/main.rs`
- `Python_Sample`
  - `robstride_dynamics/protocol.py`
  - `robstride_dynamics/bus.py`
  - `robstride_scan_info.py`
  - `robstride_scan_query_bus.py`
  - `robstride_try_control.py`
- `robstride_actuator_bridge`
  - `include/motor_control/robstride.h`
  - `src/robstride.cpp`
- `RobStride_Control`
  - `README.md`
  - `python/README.md`
  - `rust/README.md`

## 2. RobStride 协议通信类型：覆盖情况

协议常量层（`CommunicationType`）包含：

- `0 GET_DEVICE_ID`
- `1 OPERATION_CONTROL`
- `2 OPERATION_STATUS`
- `3 ENABLE`
- `4 DISABLE`
- `6 SET_ZERO_POSITION`
- `7 SET_DEVICE_ID`
- `17 READ_PARAMETER`
- `18 WRITE_PARAMETER`
- `21 FAULT_REPORT`
- `22 SAVE_PARAMETERS`
- `23 SET_BAUDRATE`
- `24 ACTIVE_REPORT`
- `25 SET_PROTOCOL`

`motorbridge` 现状：

- 已可直接发送/使用：`0/1/3/4/6/7/17/18/22`
- 已接收并处理：`2`，以及 `21`（当前按状态帧同路径处理）
- 常量存在但未暴露高层 API：`23/24/25`

结论：**主控制链路已可用且稳定，但“协议运维类高级功能”还可以继续开放。**

## 3. 当前 `motorbridge` RobStride 能力盘点

## 3.1 Rust Core (`motor_vendors/robstride`)

已支持：

- Ping（含 host 候选轮询）
- Enable / Disable
- MIT 控制
- Velocity 控制（通过参数写 `0x700A`）
- 参数读写（i8/u8/u16/u32/f32）
- 设置设备 ID（`SET_DEVICE_ID`）
- 存参（`SAVE_PARAMETERS`）
- 设零位（`SET_ZERO_POSITION`）
- 状态反馈解析（位置/速度/力矩/温度 + 标志位）

未形成独立高层 API（但协议常量已在）：

- 设置波特率 `SET_BAUDRATE` (23)
- 设置协议版本 `SET_PROTOCOL` (25)
- 主动上报策略配置 `ACTIVE_REPORT` (24)

## 3.2 CLI (`motor_cli`)

已支持模式：

- `scan`, `ping`, `enable`, `disable`, `mit`, `vel`, `read-param`, `write-param`

已支持但不是 `mode` 的方式：

- 改 ID：`--set-motor-id <id> [--store 1]`

观察：

- CLI 未单独提供 `position/current` 语义命令，但可以通过 `write-param` 写 `run_mode/loc_ref/iq_ref` 实现相同能力。
- 这意味着**协议能力并不缺，主要是“用户层命令抽象”还可增强。**

## 3.3 ABI + Python 绑定

已支持：

- `robstride_ping`
- `robstride_set_device_id`
- `robstride_get_param_*`
- `robstride_write_param_*`
- 通用 `set_zero_position` / `store_parameters` / `set_can_timeout_ms`

结论：**ABI 层对 RobStride 参数通道支持已经较全。**

## 3.4 WS Gateway

已支持：

- `robstride_ping`
- `robstride_read_param` / `robstride_write_param`
- `mit` / `vel`
- `scan` / `set_id` / `verify`
- `set_zero_position` / `store_parameters` / `set_can_timeout_ms`

结论：**远程运维场景下，WS 网关的 RobStride 能力相对完整。**

## 4. 与 `Python_Sample`、`robstride_actuator_bridge` 对比

## 4.1 对比结论（摘要）

- `Python_Sample` 强项在“扫描策略丰富（多 host、多探测策略）”。
- `robstride_actuator_bridge` 强项在“位置模式/电流模式语义函数封装”。
- `motorbridge` 强项在“多语言统一 ABI + 更工程化分层 + 多入口一致语义”。

## 4.2 明显可补齐点

1. **CLI 增加 RobStride 位置/电流快捷命令**  
虽然可用 `write-param` 实现，但用户体验不如 `--mode pos` / `--mode iq` 直观。

2. **CLI 扫描增强**  
增加 `--feedback-ids`（多 host 候选）可提升复杂总线场景命中率。

3. **暴露 23/24/25 号通信能力**  
对应波特率、主动上报、协议切换等高级运维能力。

4. **FAULT_REPORT 独立解码结构**  
当前与 OPERATION_STATUS 共路径处理，建议拆分故障码结构输出。

5. **改 ID 后自动校验链路**  
CLI 可增加 `--verify-id` for RobStride（改后自动 ping 新 ID）。

## 5. 你关心的“是否已经完整”结论

不是“功能不够用”，而是“仍有可增强空间”：

- 核心控制闭环（scan/ping/mit/vel/读写参数/改ID/设零/存参）已经可用。
- 与样例/官方协议定义对比，主要差距在：
  - 高级运维命令（23/24/25）未对外暴露
  - CLI 层缺少位置/电流的语义化快捷入口
  - 故障帧解码可更细化

如果目标是“比官方样例更强”，建议优先走这条顺序：

1. CLI 增加 `robstride pos` / `robstride current` 快捷命令（底层仍走参数写）
2. CLI 扫描支持 `--feedback-ids` 候选列表
3. 增加 `set_baudrate` / `set_protocol` / `set_active_report`
4. 故障帧单独解析与文档化

## 6. 文档与命令一致性修正（本次已修）

本次已同步修正 `rs_api_cn.md` / `rs_api.md` 中的几个不一致项：

- `run_mode` 类型修正为 `i8`（不是 `u32`）
- `write-param` 命令改为 `--param-value`（去掉不存在的 `--type/--verify`）
- 改 ID 命令改为 `--set-motor-id`（去掉不存在的 `--mode set-id --new-id`）

