# C++ Example Programs

Build from repo root:

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
```

Files:

- `cpp_wrapper_demo.cpp`: Damiao MIT loop
- `robstride_wrapper_demo.cpp`: RobStride ping / read-param / mit / vel demo
- `full_modes_demo.cpp`: Damiao full-mode control
- `pid_register_tune_demo.cpp`: Damiao tuning
- `scan_ids_demo.cpp`: Damiao scan (legacy helper)
- `pos_ctrl_demo.cpp`: Damiao position target
- `pos_repl_demo.cpp`: Damiao interactive position console

Unified scan via Rust CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```
