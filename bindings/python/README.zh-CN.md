# motorbridge Python SDK

这是基于 `motor_abi` 的 Python 绑定层。

> English version: [README.md](README.md)

## 范围

- 高层 API: `Controller`、`Motor`、`Mode`
- CLI: `motorbridge-cli`
- 厂商入口:
  - Damiao: `add_damiao_motor(...)`
  - RobStride: `add_robstride_motor(...)`

## 快速开始

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    motor = ctrl.add_damiao_motor(0x01, 0x11, "4340P")
    ctrl.enable_all()
    motor.ensure_mode(Mode.MIT, 1000)
    motor.send_mit(0.0, 0.0, 20.0, 1.0, 0.0)
    print(motor.get_state())
    motor.close()
```

RobStride 快速示例:

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    motor = ctrl.add_robstride_motor(127, 0xFF, "rs-00")
    print(motor.robstride_ping())
    print(motor.robstride_get_param_f32(0x7019))
    motor.close()
```

## CLI 示例

Damiao:

```bash
motorbridge-cli run \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride:

```bash
motorbridge-cli run \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode ping
```

RobStride 读参数:

```bash
motorbridge-cli robstride-read-param \
  --channel can0 --model rs-00 --motor-id 127 --param-id 0x7019 --type f32
```

统一扫描（双 vendor）:

```bash
motorbridge-cli scan --vendor all --channel can0 --start-id 0x01 --end-id 0xFF
```

## 示例程序

- Damiao wrapper 示例: `examples/python_wrapper_demo.py`
- RobStride wrapper 示例: `examples/robstride_wrapper_demo.py`
- Damiao 全模式示例: `examples/full_modes_demo.py`
- Damiao 扫描 / 调参 / 位置辅助:
  - `examples/scan_ids_demo.py`
  - `examples/pid_register_tune_demo.py`
  - `examples/pos_ctrl_demo.py`
  - `examples/pos_repl_demo.py`

详细见 [examples/README.md](examples/README.md)。

## 说明

- `id-dump`、`id-set` 仍是 Damiao 工作流；`scan` 支持 `damiao|robstride|all`。
- Damiao 的完整调参参考仍保留在:
  - [DAMIAO_API.md](DAMIAO_API.md)
  - [DAMIAO_API.zh-CN.md](DAMIAO_API.zh-CN.md)
