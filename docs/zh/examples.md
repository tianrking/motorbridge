# 跨语言示例

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan + Damiao 串口桥）

- Linux SocketCAN 直接使用网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- 仅 Damiao 可选串口桥链路：`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`。
- Linux SocketCAN 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。


## 索引

- Rust CLI: `motor_cli/src/main.rs`
- C ABI: `examples/c/c_abi_demo.c`
- C++ ABI: `examples/cpp/cpp_abi_demo.cpp`
- Python ctypes: `examples/python/python_ctypes_demo.py`
- 多厂商位置同步脚本: `examples/python/four_vendor_pos_sync.py`
- WS 四电机同步上位机: `examples/web/ws_quad_sync_hmi.html`
- Python SDK: `bindings/python/examples/*`
- C++ wrapper: `bindings/cpp/examples/*`

## 快速命令

```bash
cargo build -p motor_abi --release
```

Damiao Python ctypes:

```bash
python3 examples/python/python_ctypes_demo.py --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode mit
```

RobStride Python ctypes:

```bash
python3 examples/python/python_ctypes_demo.py --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping
```

RobStride C ABI:

```bash
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
LD_LIBRARY_PATH=target/release ./c_abi_demo --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode read-param --param-id 0x7019 --param-type f32
```

多厂商位置同步脚本：

```bash
python3 examples/python/four_vendor_pos_sync.py \
  damiao 0x01 damiao 0x07 myactuator 1 hightorque 1 \
  --pos 1.57 \
  --damiao-model-by-id "0x01=4340P,0x07=4310" \
  --stagger-ms 50
```

Web 上位机（单拖杆同步四电机角度）：

```bash
cargo run -p ws_gateway --release -- --bind 0.0.0.0:9002 --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
python3 -m http.server 18080
# 浏览器打开 http://127.0.0.1:18080/examples/web/ws_quad_sync_hmi.html
```
