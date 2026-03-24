# Damiao Control All In One

---

## A) CLI 部分（4个例子）

```bash
# 1) MIT（全参数）
cargo run -p motor_cli --release -- \
  --vendor damiao --transport socketcan --channel can0 --model 4340P \
  --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0.3 \
  --loop 300 --dt-ms 10

# 2) POS_VEL（位置+速度限制）
cargo run -p motor_cli --release -- \
  --vendor damiao --transport socketcan --channel can0 --model 4340P \
  --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos -2.0 --vlim 1.0 \
  --loop 200 --dt-ms 20

# 3) VEL（纯速度）
cargo run -p motor_cli --release -- \
  --vendor damiao --transport socketcan --channel can0 --model 4340P \
  --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 1.0 \
  --loop 200 --dt-ms 20

# 4) FORCE_POS（位置+力矩限幅）
cargo run -p motor_cli --release -- \
  --vendor damiao --transport socketcan --channel can0 --model 4340P \
  --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos -2.0 --vlim 1.0 --ratio 0.2 \
  --loop 300 --dt-ms 10
```

CLI 对应扫描命令（总线电机 ID 扫描）：

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --transport socketcan --channel can0 --model 4310 \
  --mode scan --start-id 1 --end-id 16
```

### A.1 参数含义 / 单位 / 常用范围（Damiao）

| 参数 | 含义 | 单位 | 常用范围（4340P/4310 调试） | 备注 |
|---|---|---|---|---|
| `--channel` | CAN 接口名 | - | `can0` / `can1` / `slcan0` | Linux 下不要写 `@bitrate` |
| `--model` | 电机型号提示 | - | `4340P` / `4310` | 建议和真实型号一致 |
| `--motor-id` | 控制帧目标 ID | - | `0x01` 起 | 与电机 ID 一致 |
| `--feedback-id` | 反馈帧 ID | - | 常见 `motor_id + 0x10` | 例如 `0x01 -> 0x11` |
| `--mode` | 控制模式 | - | `mit`/`pos-vel`/`vel`/`force-pos` | 四种核心模式 |
| `--pos` | 目标位置 | `rad` | 常见 `-3 ~ +3` 起步 | 先小范围验证方向 |
| `--vel` | 目标速度（MIT/VEL） | `rad/s` | 常见 `-2 ~ +2` 起步 | `vel` 模式是主命令 |
| `--kp` | 位置刚度（MIT） | - | 常见 `1 ~ 20` | 大了会更“硬” |
| `--kd` | 速度阻尼（MIT） | - | 常见 `0.05 ~ 1.0` | 大了更“刹车” |
| `--tau` | 前馈力矩（MIT） | `Nm` | 常见 `0.1 ~ 2.0` 起步 | 扫描里可看到该型号 `tmax` |
| `--vlim` | 速度上限（POS_VEL/FORCE_POS） | `rad/s` | 常见 `0.5 ~ 3.0` | 用于限制运动速度 |
| `--ratio` | 力矩/电流限幅比例（FORCE_POS） | `0~1` | 常见 `0.1 ~ 0.4` | 不是精确恒扭矩 |
| `--loop` | 发送周期次数 | 次 | `50 ~ 5000` | 越大持续越久 |
| `--dt-ms` | 周期时间 | `ms` | `10` / `20` | 越小刷新越快 |

### A.2 四种模式使用场景（记忆版）

1. `MIT`：最通用，`位置+速度+阻抗+前馈扭矩`一起调。适合精细调手感、做混合控制。  
2. `POS_VEL`：到某个角度，并限制速度。适合“先到位”的安全动作。  
3. `VEL`：纯速度指令。适合连续转动、速度响应测试。  
4. `FORCE_POS`：位置目标 + 力矩限幅。适合“要到位，但别死顶太狠”。  

### A.3 调参建议（实操）

1. 先用 `POS_VEL` 小速度到位，确认方向和机械限位。  
2. 再上 `MIT`，从小 `kp/kd`、小 `tau` 开始递增。  
3. `FORCE_POS` 先从 `ratio=0.1~0.2` 起步，逐步增加。  
4. 每次测试后执行文末 `disable` 停机命令。  

---

## B) ABI 部分（C/C++，4个例子）

先构建 ABI 和 demo：

```bash
cargo build -p motor_abi --release
gcc -O2 examples/c/c_abi_demo.c -I motor_abi/include -L target/release -lmotor_abi -Wl,-rpath,'$ORIGIN/../../target/release' -o examples/c/c_abi_demo
g++ -O2 examples/cpp/cpp_abi_demo.cpp -I motor_abi/include -L target/release -lmotor_abi -Wl,-rpath,'$ORIGIN/../../target/release' -o examples/cpp/cpp_abi_demo
```

```bash
# 1) C ABI: MIT
./examples/c/c_abi_demo \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0.3 --loop 100 --dt-ms 10

# 2) C ABI: POS_VEL
./examples/c/c_abi_demo \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos -2.0 --vlim 1.0 --loop 100 --dt-ms 20

# 3) C++ ABI: VEL
./examples/cpp/cpp_abi_demo \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 1.0 --loop 100 --dt-ms 20

# 4) C++ ABI: FORCE_POS
./examples/cpp/cpp_abi_demo \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos -2.0 --vlim 1.0 --ratio 0.2 --loop 100 --dt-ms 10
```

ABI（C/C++）对应扫描命令（使用 C++ 扫描示例）：

```bash
./bindings/cpp/build/scan_ids_demo \
  --channel can0 --model 4310 --start-id 1 --end-id 16
```

---

## C) ABI Python(ctypes) 部分（4个例子）

```bash
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}

# 1) MIT
python3 examples/python/python_ctypes_demo.py \
  --transport socketcan --channel can0 --vendor damiao --model 4340P \
  --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0.3 --loop 100 --dt-ms 10

# 2) POS_VEL
python3 examples/python/python_ctypes_demo.py \
  --transport socketcan --channel can0 --vendor damiao --model 4340P \
  --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos -2.0 --vlim 1.0 --loop 100 --dt-ms 20

# 3) VEL
python3 examples/python/python_ctypes_demo.py \
  --transport socketcan --channel can0 --vendor damiao --model 4340P \
  --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 1.0 --loop 100 --dt-ms 20

# 4) FORCE_POS
python3 examples/python/python_ctypes_demo.py \
  --transport socketcan --channel can0 --vendor damiao --model 4340P \
  --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos -2.0 --vlim 1.0 --ratio 0.2 --loop 100 --dt-ms 10
```

ABI Python(ctypes) 对应扫描命令（当前 ctypes demo 无 scan 子命令，配套用 Python 扫描示例）：

```bash
export PYTHONPATH=bindings/python/src
python3 bindings/python/examples/scan_ids_demo.py \
  --channel can0 --model 4310 --start-id 1 --end-id 16
```

---

## D) bindings/python 部分（4个例子）

```bash
export PYTHONPATH=bindings/python/src
```

```bash
# 1) python_wrapper_demo（MIT）
python3 bindings/python/examples/python_wrapper_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0.3 --loop 300 --dt-ms 10

# 2) full_modes_demo（POS_VEL）
python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos -2.0 --vlim 1.0 --loop 200 --dt-ms 20

# 3) full_modes_demo（VEL）
python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 1.0 --loop 200 --dt-ms 20

# 4) full_modes_demo（FORCE_POS）
python3 bindings/python/examples/full_modes_demo.py \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos -2.0 --vlim 1.0 --ratio 0.2 --loop 300 --dt-ms 10
```

bindings/python 对应扫描命令：

```bash
python3 bindings/python/examples/scan_ids_demo.py \
  --channel can0 --model 4310 --start-id 1 --end-id 16
```

---

## E) bindings/cpp 部分（4个例子）

先构建：

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}
```

```bash
# 1) cpp_wrapper_demo（MIT）
./bindings/cpp/build/cpp_wrapper_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --pos 0 --vel 0 --kp 8 --kd 0.2 --tau 0.3 --loop 100 --dt-ms 10

# 2) full_modes_demo（包含 MIT/POS_VEL/VEL/FORCE_POS）
./bindings/cpp/build/full_modes_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos -2.0 --vlim 1.0 --ratio 0.2 --loop 100 --dt-ms 10

# 3) pos_ctrl_demo（位置控制）
./bindings/cpp/build/pos_ctrl_demo \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --target-pos -2.0 --vlim 1.0 --loop 100 --dt-ms 20

# 4) scan_ids_demo（扫描）
./bindings/cpp/build/scan_ids_demo \
  --channel can0 --model 4340P --start-id 1 --end-id 16
```

bindings/cpp 对应扫描命令：

```bash
./bindings/cpp/build/scan_ids_demo \
  --channel can0 --model 4310 --start-id 1 --end-id 16
```

---

## F) 通用停机命令

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --transport socketcan --channel can0 --model 4340P \
  --motor-id 0x01 --feedback-id 0x11 --mode disable
```

---

## G) 扫描功能（4种方式）

### G.1 方式1：Rust CLI 扫描（推荐，信息最全）

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --transport socketcan --channel can0 --model 4310 \
  --mode scan --start-id 1 --end-id 16
```

典型上报字段：

- `id`
- `feedback_id`
- `model_guess`
- `limits=(pmax,vmax,tmax)`

### G.2 方式2：Python binding 扫描脚本（scan_ids_demo）

```bash
export PYTHONPATH=bindings/python/src
python3 bindings/python/examples/scan_ids_demo.py \
  --channel can0 --model 4310 --start-id 1 --end-id 16
```

典型上报字段：

- `motor-id`
- `feedback-id`
- 命中/未命中结果

### G.3 方式3：C++ binding 扫描脚本（scan_ids_demo）

```bash
./bindings/cpp/build/scan_ids_demo \
  --channel can0 --model 4310 --start-id 1 --end-id 16
```

典型上报字段：

- `motor-id`
- `feedback-id`
- 命中/未命中结果

### G.4 方式4：Python SDK CLI 扫描（motorbridge-cli）

```bash
export PYTHONPATH=bindings/python/src
python3 -m motorbridge.cli scan \
  --vendor damiao --channel can0 --start-id 0x01 --end-id 0x10
```

典型上报字段：

- `vendor`
- `id`
- 命中数量统计

### G.5 扫描信息对齐说明

1. 你要的“总线电机是否存在、ID 是多少”，4 种方式都能查。  
2. `feedback_id`：CLI / Python scan_ids_demo / C++ scan_ids_demo 可直接看到。  
3. `model_guess` 与 `limits`：以 Rust `motor_cli --mode scan` 的输出最完整，其他扫描方式通常不做完整型号猜测与极限打印。  
4. 如果要做“发现 + 精确识别 + 控制参数选型”，建议流程：
   - 先用 `G.1` 扫描
   - 再用 A/B/C/D/E 的四模式命令做控制验证
