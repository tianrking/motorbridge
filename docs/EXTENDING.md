# Extending Guide

## 1) 新增其他品牌电机（例如 Robostride）

目标：不改 `motor_core`，只新增 vendor crate。

步骤：

1. 新建 crate（示例：`motor_vendor_robostride`）
   - `Cargo.toml` 依赖 `motor_core`

2. 在新 crate 内实现这几层
   - `protocol.rs`：该品牌帧编码/解码
   - `registers.rs`：寄存器表（如果协议有寄存器）
   - `motor.rs`：`RobostrideMotor`，并实现 `motor_core::MotorDevice`
   - `controller.rs`：`RobostrideController`（facade），内部复用 `motor_core::CoreController`

3. 在 ABI 增加接入函数
   - `motor_controller_add_robostride_motor(...)`
   - 可复用 `motor_handle_*` 通用控制函数，或新增 vendor 特有控制函数

4. 更新 workspace
   - 在根 `Cargo.toml` 的 `[workspace].members` 添加新 crate

## 2) 新增同品牌不同型号（例如 Damiao 新型号）

目标：不改调度核心，仅扩型号参数。

对 Damiao：

1. 打开 `motor_vendors/damiao/src/motor.rs`
2. 在 `DAMIAO_MODELS` 里新增条目：
   - `model`
   - `pmax`
   - `vmax`
   - `tmax`
3. 保证型号字符串与业务侧传入一致（`controller.add_motor(..., model)`）

说明：
- 当前 Damiao 多型号默认按同一协议格式处理（MIT/POS_VEL/VEL/FORCE_POS + 寄存器操作）
- 型号差异主要体现在量纲映射范围（P/V/T limits）

## 3) 同品牌不同型号是否“协议一定相同”

结论：不能盲假设，必须验证。

建议验证项：

1. 帧结构是否一致
   - MIT 位打包
   - 其他模式 arbitration ID 规则
   - 系统命令字节（enable/disable/clear/zero）

2. 寄存器语义是否一致
   - RID 含义
   - 数据类型（float/u32）
   - 可读写权限
   - 控制模式寄存器值映射

3. 范围参数是否一致
   - PMAX/VMAX/TMAX
   - kp/kd 映射区间是否同默认

4. 异常状态码是否一致
   - 状态位编码和故障码定义

只要任一项不一致，就应在 vendor 内按“子协议”分支实现（例如 `damiao_v1` / `damiao_pro`）。

## 4) 当前 Damiao 实现准确性状态

已实现并与参考 Python 工程对齐的部分：

- MIT / POS_VEL / VEL / FORCE_POS 编码
- 系统命令帧（enable/disable/zero/clear）
- 反馈解码（status/pos/vel/torq/temp）
- 寄存器读写帧与回复解析
- 多型号 P/V/T 映射参数

仍建议你做实机回归确认的部分：

- 每个具体型号的寄存器兼容性（尤其扩展寄存器）
- 特定固件版本下控制模式切换行为
- 边界值（最大速度/力矩）下的稳定性与保护触发行为

## 5) 推荐最小回归清单（每新增型号/品牌都跑）

1. `enable -> ensure_mode -> MIT 零指令`
2. 正弦轨迹跟随（位置/速度/力矩打印）
3. 读寄存器 10（控制模式）并写回验证
4. 清错命令与故障恢复
5. 10~30 分钟连续运行稳定性
