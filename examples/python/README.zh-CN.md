# Python ctypes 示例

这里的 Python 示例直接通过 `ctypes` 调用 Rust ABI。

> English version: [README.md](README.md)

## 文件

- `python_ctypes_demo.py`: 统一的双 vendor 示例

覆盖范围:

- Damiao: `enable`、`disable`、`mit`、`pos-vel`、`vel`、`force-pos`
- RobStride: `ping`、`enable`、`disable`、`mit`、`vel`、`read-param`、`write-param`

## 构建与运行

```bash
cargo build -p motor_abi --release
python3 examples/python/python_ctypes_demo.py --help
```

## 示例

Damiao MIT:

```bash
python3 examples/python/python_ctypes_demo.py \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride ping:

```bash
python3 examples/python/python_ctypes_demo.py \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride 读参数:

```bash
python3 examples/python/python_ctypes_demo.py \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019 --param-type f32
```

RobStride 写参数:

```bash
python3 examples/python/python_ctypes_demo.py \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode write-param --param-id 0x700A --param-type f32 --param-value 0.2
```
