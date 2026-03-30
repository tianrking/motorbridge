# WS 四路独立滑杆 Demo（在本目录完整实现）

这个目录现在是 **WebSocket 方案**，不是 HTTP 控制后端。

> 状态说明：该 WS demo 当前为开发中版本，适合联调与验证，不建议直接视为最终生产方案。

## 已知问题（Damiao）

- 当前现场测试中，Damiao 在 WS 多会话并发下仍可能偶发抖动/回位。
- 该问题排查中；如需优先稳定运行，建议先使用 Python binding 直连控制（不走 WS）：

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
cargo build -p motor_abi --release
export PYTHONPATH=bindings/python/src
export LD_LIBRARY_PATH=$PWD/target/release:${LD_LIBRARY_PATH}
python3 bindings/python/examples/quad_vendor_pos_binding_demo.py --channel can0 --pos 0 --loop 120 --dt-ms 20
python3 bindings/python/examples/quad_vendor_pos_binding_demo.py --channel can0 --pos 3.14 --loop 120 --dt-ms 20
```

### 改动范围说明

- 本目录和相关联调改动主要在示例层（前端/脚本/README）与 `ws_gateway`。
- `motor_core` 本体未做行为改动。

- 页面：`index.html`
- 本地静态服务：`server.py`
- 电机控制链路：浏览器 `-> ws_gateway -> CAN`

默认四路目标：

- Damiao `4340P` id=`0x01` feedback=`0x11`
- Damiao `4310` id=`0x07` feedback=`0x17`
- MyActuator `X8` id=`1` feedback=`0x241`
- RobStride `rs-00` id=`127` feedback=`0xFE`

## 1) 启动 ws_gateway

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
cargo run -p ws_gateway --release -- --bind 0.0.0.0:9002 --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 50
```

## 2) 启动本目录静态服务

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
python3 examples/http_quad_control_demo/server.py --bind 127.0.0.1 --port 18081
```

## 3) 浏览器打开

```text
http://127.0.0.1:18081
```

## 4) 使用方式（四条滑杆独立控制）

1. 点击 `连接全部`
2. 点击 `使能全部`
3. 每一路各自拖动对应滑杆（可独立位置）
4. 可给某一路勾选 `自动发送`，持续保持该路目标
5. 结束时点 `停止全部` 或 `失能全部`

## 5) 稳定性建议

- 网关 `--dt-ms` 建议默认 `50`，追求更快响应再尝试 `20`
- 页面 `前端补发周期(ms,0=关)` 建议保持 `0`（让网关 continuous 自己维持）
- 页面 `错峰(ms)` 建议 `8~20`
- 若 18081 端口占用，改 `--port 18082` 后用对应地址访问
