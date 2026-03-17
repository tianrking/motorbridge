# Damiao 参数与 API 参考（Python SDK）

本文档列出 `motorbridge` Python 包对 Damiao 暴露的控制/配置接口。

> English: [DAMIAO_API.md](DAMIAO_API.md)

## 1）控制模式与实时参数

### MIT（`Mode.MIT`）

- `pos`：目标位置
- `vel`：目标速度
- `kp`：位置刚度增益
- `kd`：速度阻尼增益
- `tau`：前馈力矩

### POS_VEL（`Mode.POS_VEL`）

- `pos`：目标位置
- `vlim`：速度限制

### VEL（`Mode.VEL`）

- `vel`：目标速度

### FORCE_POS（`Mode.FORCE_POS`）

- `pos`：目标位置
- `vlim`：速度限制
- `ratio`：力矩限制比例

## 2）Python API 接口面（Controller/Motor）

Controller：

- `Controller(channel)`
- `enable_all()` / `disable_all()`
- `poll_feedback_once()`
- `shutdown()` / `close_bus()` / `close()`
- `add_damiao_motor(motor_id, feedback_id, model)`

Motor：

- 控制：`enable()`、`disable()`、`clear_error()`、`set_zero_position()`
- 模式：`ensure_mode(mode, timeout_ms=1000)`
- 指令：`send_mit()`、`send_pos_vel()`、`send_vel()`、`send_force_pos()`
- 运维：`request_feedback()`、`store_parameters()`、`set_can_timeout_ms(timeout_ms)`
- 寄存器：`write_register_f32/u32()`、`get_register_f32/u32()`
- 状态：`get_state()`

## 3）包内内置 Damiao 寄存器元数据

可以直接导入寄存器元数据：

```python
from motorbridge import (
    DAMIAO_RW_REGISTERS,
    DAMIAO_HIGH_IMPACT_RIDS,
    DAMIAO_PROTECTION_RIDS,
    get_damiao_register_spec,
    RID_CTRL_MODE,
)

print(DAMIAO_RW_REGISTERS[22])
print(DAMIAO_HIGH_IMPACT_RIDS)
print(get_damiao_register_spec(RID_CTRL_MODE))
```

打印当前导出的全部可调寄存器与范围：

```python
from motorbridge import DAMIAO_RW_REGISTERS

for rid in sorted(DAMIAO_RW_REGISTERS):
    spec = DAMIAO_RW_REGISTERS[rid]
    print(f"rid={rid:>2} {spec.variable:<10} type={spec.data_type} range={spec.range_str} desc={spec.description}")
```

导出常量：

- ID/模式相关：`RID_MST_ID`、`RID_ESC_ID`、`RID_CTRL_MODE`、`RID_TIMEOUT`
- 模式值：`MODE_MIT`、`MODE_POS_VEL`、`MODE_VEL`、`MODE_FORCE_POS`

## 4）强影响寄存器（优先）

- `21 PMAX`、`22 VMAX`、`23 TMAX`
- `25 KP_ASR`、`26 KI_ASR`、`27 KP_APR`、`28 KI_APR`
- `4 ACC`、`5 DEC`、`6 MAX_SPD`、`9 TIMEOUT`

## 5）保护相关寄存器

- `0 UV_Value`、`2 OT_Value`、`3 OC_Value`、`29 OV_Value`

## 6）推荐调参流程

1. 先读旧值（`get_register_*`）
2. 写新值（`write_register_*`）
3. 回读确认
4. `store_parameters()` 持久化

## 7）完整参数总表

完整调参与命令示例见：

- [`examples/dm_api.md`](../examples/dm_api.md)
- [`examples/dm_api_cn.md`](../examples/dm_api_cn.md)
