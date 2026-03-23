# C++ 示例程序

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan）

- Linux 下直接使用 SocketCAN 网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- Linux 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 在 SocketCAN 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。


在仓库根目录构建:

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
```

文件说明:

- `cpp_wrapper_demo.cpp`: Damiao MIT 循环
- `robstride_wrapper_demo.cpp`: RobStride 的 ping / read-param / mit / vel 示例
- `full_modes_demo.cpp`: Damiao 全模式控制
- `pid_register_tune_demo.cpp`: Damiao 调参
- `scan_ids_demo.cpp`: Damiao 扫描（历史辅助）
- `pos_ctrl_demo.cpp`: Damiao 目标位置
- `pos_repl_demo.cpp`: Damiao 交互式位置控制台

通过 Rust CLI 统一扫描:

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```
