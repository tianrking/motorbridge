# Web 上位机示例

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
cargo run -p ws_gateway --release -- --bind 0.0.0.0:9002 --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 20

# 终端 B：在仓库根目录启动静态服务
python3 -m http.server 18080

# 浏览器打开
http://127.0.0.1:18080/examples/web/ws_quad_sync_hmi.html
```

### 稳定性建议

- 页面采用每个电机一个 WS 会话，减少频繁 `set_target` 的切换开销。
- 若某一路偶发断连：
  - 将网关 `--dt-ms` 从 `20` 提高到 `50`
  - 页面 `发送周期(ms)` 设为 `60~100`
  - 页面 `错峰(ms)` 设为 `8~20`
