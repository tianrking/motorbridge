# Web 上位机示例

CAN 联调与排障文档： [../../docs/zh/can_debugging.md](../../docs/zh/can_debugging.md)

> 状态说明：本页涉及的 WS 上位机方案仍在持续开发与调优中（尤其是 Damiao 多会话并发场景）。

## 已知问题（Damiao + WS 场景）

- 在部分现场总线负载下，Damiao 通过 `ws_gateway` 多会话并发控制时可能出现：
  - 偶发抖动/回位
  - `socketcan write failed: No buffer space available (os error 105)`
- 若你当前项目优先稳定性，建议暂时改用 Python binding 直连控制（不经过 WS）：

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
cargo build -p motor_abi --release
export PYTHONPATH=bindings/python/src
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}
python3 bindings/python/examples/quad_vendor_pos_binding_demo.py --channel can0 --pos 0 --loop 120 --dt-ms 20
python3 bindings/python/examples/quad_vendor_pos_binding_demo.py --channel can0 --pos 3.14 --loop 120 --dt-ms 20
```

### 改动范围说明

- 当前仓库内相关修复主要在示例层与网关层：
  - `examples/*`（页面、脚本、README）
  - `integrations/ws_gateway/*`（WS 网关负载与轮询策略）
- `motor_core` 本体未做行为改动。

若仍希望使用 WS 上位机交互，但绕开 `ws_gateway`，可试 Python binding WS bridge：

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
cargo build -p motor_abi --release
export PYTHONPATH=bindings/python/src
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}
python3 bindings/python/examples/quad_vendor_binding_ws_demo.py --bind 127.0.0.1 --port 9010 --channel can0 --dt-ms 20
# 浏览器打开：
# file:///home/w0x7ce/Downloads/dm_candrive/rust_dm/bindings/python/examples/quad_vendor_binding_ws_demo.html
```

## ws_quad_sync_hmi.html

基于 `ws_gateway` 的单拖杆四电机同角度同步控制页面。

默认目标设备：

- Damiao `0x01`（`4340P`，反馈 `0x11`）
- Damiao `0x07`（`4310`，反馈 `0x17`）
- MyActuator `1`（`X8`，反馈 `0x241`）
- HighTorque `1`（`hightorque`，反馈 `0x01`）

### 运行

```bash
# 终端 A：启动网关
cargo run -p ws_gateway --release -- --bind 127.0.0.1:9002 --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 20

# 终端 B：在仓库根目录启动静态服务
python3 -m http.server 18080

# 浏览器打开
http://127.0.0.1:18080/examples/web/ws_quad_sync_hmi.html
```

### 稳定性建议

- 页面采用每个电机一个 WS 会话，减少频繁 `set_target` 的切换开销。
- 位置命令默认 `continuous=true`，发送一次后网关会持续保位；需要释放时请点“停止全部”或“失能全部”。
- 若某一路偶发断连：
  - 将网关 `--dt-ms` 从 `20` 提高到 `50`
  - 页面 `发送周期(ms)` 设为 `60~100`
  - 页面 `错峰(ms)` 设为 `8~20`

## ws_quad_sync_hmi_rs.html

在 `ws_quad_sync_hmi.html` 基础上，仅将第 4 路从 HighTorque 替换为 RobStride，其它三路保持不变。

默认目标设备：

- Damiao `0x01`（`4340P`，反馈 `0x11`）
- Damiao `0x07`（`4310`，反馈 `0x17`）
- MyActuator `1`（`X8`，反馈 `0x241`）
- RobStride `127`（`rs-00`，反馈 `0xFE`）

### 运行

```bash
# 终端 A：启动网关
cargo run -p ws_gateway --release -- --bind 127.0.0.1:9002 --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 20

# 终端 B：在仓库根目录启动静态服务
python3 -m http.server 18080

# 浏览器打开（RobStride 版本）
http://127.0.0.1:18080/examples/web/ws_quad_sync_hmi_rs.html
```
