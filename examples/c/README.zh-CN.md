# C ABI 示例

本目录展示如何在 C 中调用 Rust `motor_abi`。

> English version: [README.md](README.md)

## 文件

- `c_abi_demo.c`：统一多模式命令行示例（`enable/disable/mit/pos-vel/vel/force-pos`）

## 构建

在项目根目录（`rust_dm`）执行：

```bash
cargo build -p motor_abi --release
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
```

运行：

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo --help
```

运行前请先配置 CAN：

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

## 完整命令

使能：

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100 --print-state 1
```

失能：

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100 --print-state 1
```

MIT：

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 200 --dt-ms 20 --print-state 1
```

POS_VEL：

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 \
  --ensure-mode 1 --ensure-timeout-ms 1000 --ensure-strict 0 \
  --loop 300 --dt-ms 20 --print-state 1
```

VEL：

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 --ensure-mode 1 --loop 100 --dt-ms 20 --print-state 1
```

FORCE_POS：

```bash
LD_LIBRARY_PATH=target/release ./c_abi_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 \
  --ensure-mode 1 --loop 100 --dt-ms 20 --print-state 1
```
