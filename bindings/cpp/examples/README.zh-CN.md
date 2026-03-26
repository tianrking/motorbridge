# C++ 示例程序

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan + Damiao 串口桥）

- Linux SocketCAN 直接使用网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- 仅 Damiao 可选串口桥链路：`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`。
- Damiao 串口桥完整接口与命令模板见 `motor_cli/README.zh-CN.md` 第 `3.6` 节（英文见 `motor_cli/README.md`）。
- Linux SocketCAN 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。

## Damiao 置零顺序说明

- 推荐顺序：`disable -> set_zero_position -> enable -> ensure_mode -> control`。
- `set_zero_position` 由核心层防护，要求电机处于失能状态。


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
