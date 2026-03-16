# C++ ABI Examples

C++ examples for calling Rust `motor_abi` from C++.

> Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Files

- `cpp_abi_demo.cpp`: unified multi-mode CLI demo (`enable/disable/mit/pos-vel/vel/force-pos`)

## Build

From project root (`rust_dm`):

```bash
cargo build -p motor_abi --release
g++ -std=c++17 examples/cpp/cpp_abi_demo.cpp -I motor_abi/include -L target/release -lmotor_abi -o cpp_abi_demo
```

Run:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo --help
```

## Full Commands

Enable:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100 --print-state 1
```

Disable:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100 --print-state 1
```

MIT:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 200 --dt-ms 20 --print-state 1
```

POS_VEL:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 300 --dt-ms 20 --print-state 1
```

VEL:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 --ensure-mode 1 --loop 100 --dt-ms 20 --print-state 1
```

FORCE_POS:

```bash
LD_LIBRARY_PATH=target/release ./cpp_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 \
  --ensure-mode 1 --loop 100 --dt-ms 20 --print-state 1
```
