# motorbridge C++ 绑定（RAII 封装）

这是基于稳定 C ABI（`motor_abi`）的一层轻量 C++ 封装。

> English version: [README.md](README.md)

## 设计目标

- 保持 ABI 稳定性（底层仍全部调用 `motor_abi`）
- 提供现代 C++ 体验（RAII + 异常 + 强类型接口）
- 与 Python SDK 核心能力对齐

## 与 Python 的能力对齐

`motorbridge::Controller`：

- `enable_all()`、`disable_all()`
- `poll_feedback_once()`
- `shutdown()`、`close_bus()`
- `add_damiao_motor(motor_id, feedback_id, model)`

`motorbridge::Motor`：

- 控制：`enable()`、`disable()`、`clear_error()`、`set_zero_position()`
- 模式：`ensure_mode(mode, timeout_ms)`
- 指令：`send_mit()`、`send_pos_vel()`、`send_vel()`、`send_force_pos()`
- 运维：`request_feedback()`、`set_can_timeout_ms()`、`store_parameters()`
- 寄存器：`write_register_f32/u32()`、`get_register_f32/u32()`
- 状态：`get_state()`（`std::optional<State>`）

Damiao 参数元数据（与 Python 导出对齐）：

- `DAMIAO_RW_REGISTERS`
- `DAMIAO_HIGH_IMPACT_RIDS`
- `DAMIAO_PROTECTION_RIDS`
- `get_damiao_register_spec(rid)`

## 目录结构

- 头文件：`include/motorbridge/motorbridge.hpp`
- CMake 包配置：`CMakeLists.txt`、`cmake/motorbridge-config.cmake.in`
- 封装示例：`examples/cpp_wrapper_demo.cpp`
- 总线快速扫描示例：`examples/scan_ids_demo.cpp`
- 单次目标位置控制示例：`examples/pos_ctrl_demo.cpp`
- 交互式位置控制台示例：`examples/pos_repl_demo.cpp`

## 无需 Rust 的安装方式（推荐用户）

用户机器上不需要安装 Rust，直接从 GitHub Releases 下载预编译产物即可。

### 1）Linux x86_64：安装 `.deb`（推荐）

下载：

- `motorbridge-abi-<version>-linux-x86_64.deb`

安装：

```bash
sudo dpkg -i motorbridge-abi-<version>-linux-x86_64.deb
```

安装后默认路径：

- 头文件：`/usr/include/motorbridge/motorbridge.hpp`、`/usr/include/motor_abi.h`
- 库文件：`/usr/lib/libmotor_abi.so`、`/usr/lib/libmotor_abi.a`
- CMake 包：`/usr/lib/cmake/motorbridge/*`

快速检查：

```bash
dpkg -L motorbridge-abi | grep -E "motorbridge.hpp|motor_abi.h|libmotor_abi|cmake/motorbridge"
```

### 2）Linux aarch64 / Windows x86_64：使用压缩包

下载并解压：

- Linux aarch64：`motorbridge-abi-<version>-linux-aarch64.tar.gz`
- Windows x86_64：`motorbridge-abi-<version>-windows-x86_64.zip`

解压后建议保持目录结构：

- `<prefix>/include/motorbridge/motorbridge.hpp`
- `<prefix>/include/motor_abi.h`
- `<prefix>/lib/*`（包含 `libmotor_abi.so` 或 `motor_abi.dll/.lib`）
- `<prefix>/lib/cmake/motorbridge/*`

然后在 CMake 里通过 `CMAKE_PREFIX_PATH` 指向 `<prefix>`。

## 用户侧 `find_package` 调用

`CMakeLists.txt`：

```cmake
find_package(motorbridge CONFIG REQUIRED)
add_executable(app main.cpp)
target_link_libraries(app PRIVATE motorbridge::cpp)
```

构建：

- Linux `.deb` 安装后：

```bash
cmake -S . -B build
cmake --build build -j
```

- 使用手动解压包：

```bash
cmake -S . -B build -DCMAKE_PREFIX_PATH=/path/to/motorbridge-prefix
cmake --build build -j
```

最小编译/链接自检（`.deb` 安装后）：

```bash
cat > CMakeLists.txt <<'CMAKE'
cmake_minimum_required(VERSION 3.16)
project(mb_check LANGUAGES CXX)
find_package(motorbridge CONFIG REQUIRED)
add_executable(mb_check main.cpp)
target_link_libraries(mb_check PRIVATE motorbridge::cpp)
CMAKE
cat > main.cpp <<'CPP'
#include "motorbridge/motorbridge.hpp"
int main() { return motorbridge::get_damiao_register_spec(motorbridge::RID_CTRL_MODE) ? 0 : 1; }
CPP
cmake -S . -B build && cmake --build build -j && ./build/mb_check
```

运行时库注意事项：

- Linux：确保系统能找到 `libmotor_abi.so`（`ldconfig`、`LD_LIBRARY_PATH` 或 rpath）。
- Windows：确保 `motor_abi.dll` 在可执行文件同目录，或其路径已加入 `PATH`。

## 运行前 CAN 配置（Linux）

如果出现 `socketcan write failed: Network is down (os error 100)`，说明 `can0` 未正确启动。

建议命令（以 1Mbps 为例）：

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

可选总线观察（需要 `can-utils`）：

```bash
candump can0
```

若未安装：

```bash
sudo apt install can-utils
```

## 最小调用示例

```cpp
#include "motorbridge/motorbridge.hpp"

int main() {
  motorbridge::Controller ctrl("can0");
  auto m = ctrl.add_damiao_motor(0x01, 0x11, "4340P");
  ctrl.enable_all();
  m.ensure_mode(motorbridge::Mode::MIT, 1000);
  m.send_mit(0.0f, 0.0f, 20.0f, 1.0f, 0.0f);
  auto st = m.get_state();
  ctrl.shutdown();
  return 0;
}
```

## 本地构建示例（开发者）

在仓库根目录执行：

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/cpp_wrapper_demo
```

## 实用 C++ 小示例

先构建：

```bash
cargo build -p motor_abi --release
cmake -S bindings/cpp -B bindings/cpp/build \
  -DMOTORBRIDGE_ABI_LIBRARY=$PWD/target/release/libmotor_abi.so
cmake --build bindings/cpp/build -j
```

1）总线快速扫描：

```bash
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/scan_ids_demo \
  --channel can0 --model 4310 --start-id 0x01 --end-id 0xFF --feedback-base 0x10 --timeout-ms 120
```

2）单次目标位置控制（POS_VEL）：

```bash
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/pos_ctrl_demo \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --target-pos 3.14 --vlim 1.5 --loop 300 --dt-ms 20
```

3）交互式位置控制台：

```bash
LD_LIBRARY_PATH=target/release ./bindings/cpp/build/pos_repl_demo \
  --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 --vlim 1.5
```

启动后直接输入 `1`、`3.14` 回车即可发送目标位置。
