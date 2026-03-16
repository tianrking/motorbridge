# 示例索引

本目录用于快速查找：

- 不同控制模式需要哪些参数
- 不同语言（CLI / Python / C / C++）对应调用哪个方法
- 每种调用方式如何运行

> English version: [README.md](README.md)

## 文件索引

- Rust CLI：`motor_cli/src/main.rs`
- Rust 原生示例：
  - `motor_vendors/damiao/examples/test_4340.rs`
  - `motor_vendors/damiao/examples/test_4340p.rs`
- C ABI 示例：`examples/c/c_abi_demo.c`
- C++ ABI 示例：`examples/cpp/cpp_abi_demo.cpp`
- Python ctypes 示例：`examples/python/python_ctypes_demo.py`
- ABI 头文件：`motor_abi/include/motor_abi.h`

## 通用设备参数

- `channel`：CAN 接口名（例如 `can0`）
- `model`：电机型号（例如 `4340`, `4340P`, `4310`, `8006`）
- `motor_id`：命令 ID（`ESC_ID`）
- `feedback_id`：反馈 ID（`MST_ID`）

## 控制模式与参数

| 模式 | 含义 | 必填参数 |
|---|---|---|
| `mit` | 位置 + 速度 + 刚度 + 阻尼 + 前馈力矩 | `pos`, `vel`, `kp`, `kd`, `tau` |
| `pos-vel` | 位置控制（带速度限制） | `pos`, `vlim` |
| `vel` | 速度控制 | `vel` |
| `force-pos` | 力位混合 | `pos`, `vlim`, `ratio` |

## 方法映射（CLI / Python / C / C++）

| 模式 | CLI | Python ctypes | C ABI | C++ ABI |
|---|---|---|---|---|
| MIT | `--mode mit --pos --vel --kp --kd --tau` | `motor_handle_send_mit(...)` | `motor_handle_send_mit(...)` | `motor_handle_send_mit(...)` |
| POS_VEL | `--mode pos-vel --pos --vlim` | `motor_handle_send_pos_vel(...)` | `motor_handle_send_pos_vel(...)` | `motor_handle_send_pos_vel(...)` |
| VEL | `--mode vel --vel` | `motor_handle_send_vel(...)` | `motor_handle_send_vel(...)` | `motor_handle_send_vel(...)` |
| FORCE_POS | `--mode force-pos --pos --vlim --ratio` | `motor_handle_send_force_pos(...)` | `motor_handle_send_force_pos(...)` | `motor_handle_send_force_pos(...)` |

模式设置（所有语言通用）：

- 切换模式：`motor_handle_ensure_mode(motor, mode, timeout_ms)`
- 模式值：`1=MIT`, `2=POS_VEL`, `3=VEL`, `4=FORCE_POS`

## 快速运行

### 1) CLI（推荐）

```bash
cargo run -p motor_cli -- --help
```

MIT：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 30 --kd 1 --tau 0 --loop 200 --dt-ms 20
```

位置模式（POS_VEL）：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 1.2 --vlim 2.0 --loop 100 --dt-ms 20
```

### 2) Python ctypes

```bash
cargo build -p motor_abi --release
python3 examples/python/python_ctypes_demo.py --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11
```

### 3) C

```bash
cargo build -p motor_abi --release
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
LD_LIBRARY_PATH=target/release ./c_abi_demo can0 4340P 0x01 0x11
```

### 4) C++

```bash
cargo build -p motor_abi --release
g++ -std=c++17 examples/cpp/cpp_abi_demo.cpp -I motor_abi/include -L target/release -lmotor_abi -o cpp_abi_demo
LD_LIBRARY_PATH=target/release ./cpp_abi_demo can0 4340P 0x01 0x11
```

## 常用运行时接口

- feedback poll：`motor_controller_poll_feedback_once(...)`
- 读取状态：`motor_handle_get_state(...)`
- 寄存器读写：
  - `motor_handle_write_register_f32/u32(...)`
  - `motor_handle_get_register_f32/u32(...)`
- 运维接口：
  - `motor_handle_clear_error(...)`
  - `motor_handle_set_zero_position(...)`
  - `motor_handle_store_parameters(...)`
  - `motor_handle_set_can_timeout_ms(...)`
