# C++ 示例程序

本目录包含基于 `motorbridge/motorbridge.hpp` 的实用示例。

- `cpp_wrapper_demo.cpp`：MIT 循环最小示例
- `full_modes_demo.cpp`：全模式全参数示例（`enable/disable/mit/pos-vel/vel/force-pos`）
- `pid_register_tune_demo.cpp`：PID 与高影响寄存器调参（读/写/回读校验）
- `scan_ids_demo.cpp`：CAN ID 扫描
- `pos_ctrl_demo.cpp`：单次目标位置命令（POS_VEL）
- `pos_repl_demo.cpp`：交互式位置控制台

在仓库根目录构建：

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
```

运行示例（全模式）：

```bash
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/full_modes_demo --help
```

运行示例（PID 调参）：

```bash
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/pid_register_tune_demo --help
```
