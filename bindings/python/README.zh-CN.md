# motorbridge Python SDK

用于调用 `motorbridge` Rust ABI 的 Python 包。

> English: [README.md](README.md)

## 安装（本地开发）

在仓库根目录执行：

```bash
cd bindings/python
pip install -e .
```

运行前先构建一次 Rust ABI：

```bash
cd ../../
cargo build -p motor_abi --release
```

## 快速使用

```python
from motorbridge import Controller, Mode

with Controller("can0") as ctrl:
    m = ctrl.add_damiao_motor(0x01, 0x11, "4340P")
    ctrl.enable_all()
    m.ensure_mode(Mode.MIT, 1000)
    m.send_mit(0.0, 0.0, 20.0, 1.0, 0.0)
    print(m.get_state())
    m.close()
```

## 命令行

```bash
motorbridge-cli --help
```

示例：

```bash
motorbridge-cli \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 300 --dt-ms 20
```

## 动态库加载顺序

1. 环境变量 `MOTORBRIDGE_LIB`
2. 包内 `motorbridge/lib/*`
3. 仓库 `target/release/*`
4. `ctypes.util.find_library("motor_abi")`
