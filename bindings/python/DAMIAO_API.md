# Damiao Parameter and API Reference (Python SDK)

This document lists Damiao control/configuration interfaces exposed by the `motorbridge` Python package.

> 中文版: [DAMIAO_API.zh-CN.md](DAMIAO_API.zh-CN.md)

## 1) Control Modes and Runtime Parameters

### MIT (`Mode.MIT`)

- `pos`: target position
- `vel`: target velocity
- `kp`: position stiffness gain
- `kd`: velocity damping gain
- `tau`: feedforward torque

### POS_VEL (`Mode.POS_VEL`)

- `pos`: target position
- `vlim`: velocity limit

### VEL (`Mode.VEL`)

- `vel`: target velocity

### FORCE_POS (`Mode.FORCE_POS`)

- `pos`: target position
- `vlim`: velocity limit
- `ratio`: torque limit ratio

## 2) Python API Surface (Controller/Motor)

Controller:

- `Controller(channel)`
- `enable_all()` / `disable_all()`
- `poll_feedback_once()`
- `shutdown()` / `close_bus()` / `close()`
- `add_damiao_motor(motor_id, feedback_id, model)`

Motor:

- control: `enable()`, `disable()`, `clear_error()`, `set_zero_position()`
  - project rule: call `disable()` before `set_zero_position()`
  - Python API has no `ms` argument for set-zero; core applies fixed `20ms` settle internally
- mode: `ensure_mode(mode, timeout_ms=1000)`
- command: `send_mit()`, `send_pos_vel()`, `send_vel()`, `send_force_pos()`
- ops: `request_feedback()`, `store_parameters()`, `set_can_timeout_ms(timeout_ms)`
- register: `write_register_f32/u32()`, `get_register_f32/u32()`
- state: `get_state()`

## 3) Built-in Damiao Register Metadata (in package)

You can import built-in register metadata directly:

```python
from motorbridge import (
    DAMIAO_RW_REGISTERS,
    DAMIAO_HIGH_IMPACT_RIDS,
    DAMIAO_PROTECTION_RIDS,
    get_damiao_register_spec,
    RID_CTRL_MODE,
)

print(DAMIAO_RW_REGISTERS[22])  # VMAX spec
print(DAMIAO_HIGH_IMPACT_RIDS)
print(get_damiao_register_spec(RID_CTRL_MODE))
```

Print all currently exported tunable registers with ranges:

```python
from motorbridge import DAMIAO_RW_REGISTERS

for rid in sorted(DAMIAO_RW_REGISTERS):
    spec = DAMIAO_RW_REGISTERS[rid]
    print(f"rid={rid:>2} {spec.variable:<10} type={spec.data_type} range={spec.range_str} desc={spec.description}")
```

Exported constants:

- ID/mode related: `RID_MST_ID`, `RID_ESC_ID`, `RID_CTRL_MODE`, `RID_TIMEOUT`
- mode values: `MODE_MIT`, `MODE_POS_VEL`, `MODE_VEL`, `MODE_FORCE_POS`

## 4) High-impact RIDs (Priority)

- `21 PMAX`, `22 VMAX`, `23 TMAX`
- `25 KP_ASR`, `26 KI_ASR`, `27 KP_APR`, `28 KI_APR`
- `4 ACC`, `5 DEC`, `6 MAX_SPD`, `9 TIMEOUT`

## 5) Protection RIDs

- `0 UV_Value`, `2 OT_Value`, `3 OC_Value`, `29 OV_Value`

## 6) Recommended Tuning Workflow

1. Read old value (`get_register_*`)
2. Write new value (`write_register_*`)
3. Read back for verification
4. Persist with `store_parameters()`

## 7) Full Damiao Tuning Table

For a full operation/tuning table and command examples, see:

- [`motor_cli/DAMIAO_API.md`](../../motor_cli/DAMIAO_API.md)
- [`motor_cli/DAMIAO_API.zh-CN.md`](../../motor_cli/DAMIAO_API.zh-CN.md)
