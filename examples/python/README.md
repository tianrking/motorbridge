# Python ctypes Examples

Python demos that call the Rust ABI directly through `ctypes`.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## File

- `python_ctypes_demo.py`: unified two-vendor demo

Vendor coverage:

- Damiao: `enable`, `disable`, `mit`, `pos-vel`, `vel`, `force-pos`
- RobStride: `ping`, `enable`, `disable`, `mit`, `vel`, `read-param`, `write-param`

## Build and Run

```bash
cargo build -p motor_abi --release
python3 examples/python/python_ctypes_demo.py --help
```

## Examples

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

RobStride read parameter:

```bash
python3 examples/python/python_ctypes_demo.py \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019 --param-type f32
```

RobStride write parameter:

```bash
python3 examples/python/python_ctypes_demo.py \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode write-param --param-id 0x700A --param-type f32 --param-value 0.2
```
