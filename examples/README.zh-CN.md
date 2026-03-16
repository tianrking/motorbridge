# 示例索引

本目录用于快速查找：

- 每种控制模式的完整参数
- 不同语言（CLI / Python / C / C++）的方法映射
- 可直接运行的命令（包含单独使能/失能）

> English version: [README.md](README.md)

## 文件索引

- Rust CLI：`motor_cli/src/main.rs`
- Rust 原生示例：
  - `motor_vendors/damiao/examples/test_4340.rs`
  - `motor_vendors/damiao/examples/test_4340p.rs`
- C ABI 示例：`examples/c/c_abi_demo.c`
- C 示例说明（英文）：`examples/c/README.md`
- C 示例说明（中文）：`examples/c/README.zh-CN.md`
- C++ ABI 示例：`examples/cpp/cpp_abi_demo.cpp`
- C++ 示例说明（英文）：`examples/cpp/README.md`
- C++ 示例说明（中文）：`examples/cpp/README.zh-CN.md`
- Python ctypes 示例：`examples/python/python_ctypes_demo.py`
- Python 示例说明（英文）：`examples/python/README.md`
- Python 示例说明（中文）：`examples/python/README.zh-CN.md`
- Python SDK 包（推荐正式集成）：`bindings/python`
- ABI 头文件：`motor_abi/include/motor_abi.h`

## 通用设备参数

| 参数 | 说明 | 默认值 |
|---|---|---|
| `channel` | CAN 接口名 | `can0` |
| `model` | 电机型号（如 `4340`, `4340P`, `4310`） | `4340` |
| `motor-id` | 命令 ID（`ESC_ID`） | `0x01` |
| `feedback-id` | 反馈 ID（`MST_ID`） | `0x11` |
| `loop` | 发送循环次数 | `1` |
| `dt-ms` | 发送周期（毫秒） | `20` |
| `ensure-mode` | 发送前是否确保控制模式（`1/0`） | `1` |
| `verify-model` | 通过读取 `PMAX/VMAX/TMAX` 校验 `--model`（`1/0`） | `1` |
| `verify-timeout-ms` | 型号校验寄存器读取超时（毫秒） | `500` |
| `verify-tol` | `PMAX/VMAX/TMAX` 比较绝对容差 | `0.2` |

## 各模式完整参数（CLI）

### 1) 单独使能

- 模式：`--mode enable`
- 可选：`--loop`, `--dt-ms`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 1
```

### 2) 单独失能

- 模式：`--mode disable`
- 可选：`--loop`, `--dt-ms`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 1
```

### 型号握手校验（推荐）

- 默认开启：`--verify-model 1`
- 读取寄存器：`rid=21/22/23`（`PMAX/VMAX/TMAX`）
- 不匹配时：CLI 直接退出并给出建议型号

通过示例（型号正确）：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 5 --dt-ms 100
```

失败示例（故意写错型号）：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 5 --dt-ms 100
```

### 3) MIT 模式

- 模式：`--mode mit`
- 必填控制参数：`--pos --vel --kp --kd --tau`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 30 --kd 1 --tau 0 --loop 200 --dt-ms 20
```

### 4) POS_VEL 模式（位置速度）

- 模式：`--mode pos-vel`
- 必填控制参数：`--pos --vlim`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 1.2 --vlim 2.0 --loop 100 --dt-ms 20
```

目标位置测试（以速度限制到达 `3.10 rad`）：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 300 --dt-ms 20
```

### 5) VEL 模式

- 模式：`--mode vel`
- 必填控制参数：`--vel`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 --loop 100 --dt-ms 20
```

### 6) FORCE_POS 模式

- 模式：`--mode force-pos`
- 必填控制参数：`--pos --vlim --ratio`

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 --loop 100 --dt-ms 20
```

## 方法映射（CLI / Python / C / C++）

| 动作 | CLI | Python ctypes / C / C++ ABI |
|---|---|---|
| 使能 | `--mode enable` | `motor_handle_enable(...)` |
| 失能 | `--mode disable` | `motor_handle_disable(...)` |
| MIT 发送 | `--mode mit ...` | `motor_handle_send_mit(...)` |
| POS_VEL 发送 | `--mode pos-vel ...` | `motor_handle_send_pos_vel(...)` |
| VEL 发送 | `--mode vel ...` | `motor_handle_send_vel(...)` |
| FORCE_POS 发送 | `--mode force-pos ...` | `motor_handle_send_force_pos(...)` |
| 确保模式 | `--ensure-mode 1` | `motor_handle_ensure_mode(...)` |

ABI 模式值：

- `1=MIT`, `2=POS_VEL`, `3=VEL`, `4=FORCE_POS`

## 跨语言快速运行

### Python ctypes

```bash
cargo build -p motor_abi --release
python3 examples/python/python_ctypes_demo.py --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11
```

### C

```bash
cargo build -p motor_abi --release
cc examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -o c_abi_demo
LD_LIBRARY_PATH=target/release ./c_abi_demo can0 4340P 0x01 0x11
```

### C++

```bash
cargo build -p motor_abi --release
g++ -std=c++17 examples/cpp/cpp_abi_demo.cpp -I motor_abi/include -L target/release -lmotor_abi -o cpp_abi_demo
LD_LIBRARY_PATH=target/release ./cpp_abi_demo can0 4340P 0x01 0x11
```

## 常用运行时接口

- `motor_controller_poll_feedback_once(...)`
- `motor_handle_get_state(...)`
- `motor_handle_write_register_f32/u32(...)`
- `motor_handle_get_register_f32/u32(...)`
- `motor_handle_clear_error(...)`
- `motor_handle_set_zero_position(...)`
- `motor_handle_store_parameters(...)`
- `motor_handle_set_can_timeout_ms(...)`

## 重要行为说明

- `motor_cli` 的 `--mode enable` / `--mode disable` 现在在退出时仅关闭本地总线会话。
- 不会再通过 `shutdown` 隐式自动 `disable` 电机。
