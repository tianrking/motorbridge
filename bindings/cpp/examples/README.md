# C++ Example Programs

This folder contains practical demos based on `motorbridge/motorbridge.hpp`.

- `cpp_wrapper_demo.cpp`: minimal MIT loop demo
- `full_modes_demo.cpp`: full-mode + full-parameter demo (`enable/disable/mit/pos-vel/vel/force-pos`)
- `pid_register_tune_demo.cpp`: PID and high-impact register tuning (read/write/verify)
- `scan_ids_demo.cpp`: CAN ID scan
- `pos_ctrl_demo.cpp`: one-shot position target command (POS_VEL)
- `pos_repl_demo.cpp`: interactive position console

Build from repo root:

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
```

Run example (full modes):

```bash
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/full_modes_demo --help
```

Run example (PID tuning):

```bash
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/pid_register_tune_demo --help
```
