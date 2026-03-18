# C ABI 示例

这里是直接调用 `motor_abi` 的 C 示例。

> English version: [README.md](README.md)

## 文件

- `c_abi_demo.c`: 同时支持两个 vendor 的统一示例

覆盖范围:

- Damiao: `enable`、`disable`、`mit`、`pos-vel`、`vel`、`force-pos`
- RobStride: `ping`、`enable`、`disable`、`mit`、`vel`、`read-param`、`write-param`

## 构建

```bash
cargo build -p motor_abi --release
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
LD_LIBRARY_PATH=target/release ./c_abi_demo --help
```

## 示例

Damiao MIT:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```

RobStride ping:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride 读取位置参数:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019 --param-type f32
```

RobStride 低增益 MIT:

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0 --loop 20 --dt-ms 50
```
