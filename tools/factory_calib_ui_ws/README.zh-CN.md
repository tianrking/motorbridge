# MotorBridge Assistant（factory_calib_ui_ws，React 版）

基于 WebSocket 的工厂上位机（纯前端）。

- 前端：React + Vite
- 后端：`ws_gateway`
- 浏览器调用 WS 指令：`set_target` / `scan` / `set_id` / `verify` / `enable` / `disable` / `mit` / `pos_vel` / `vel` / `force_pos` / `stop`

## 1）安装依赖

```bash
pip install -U motorbridge
cd tools/factory_calib_ui_ws
npm install
```

安装后可直接使用 `motorbridge-gateway`（无需额外配置 `MOTORBRIDGE_WS_GATEWAY_BIN` 环境变量）。
若选择 CAN 路径（`auto/socketcan/socketcanfd`），命令行会在启动前做平台提示：
- Linux：提示 `canX/slcanX` 初始化与 `ip link up`
- Windows：提示安装 PEAK PCAN 驱动与 `PCANBasic.dll`
- macOS：提示安装 PCBUSB 运行时
`dm-serial` 路径不触发上述 CAN 依赖检查。

## 2）选择传输链路并启动 `ws_gateway`

以下示例统一使用：

```bash
--bind 127.0.0.1:9002
```

安全说明：

- 本地联调建议保持 `127.0.0.1:9002`。
- 若需暴露到局域网/公网（`0.0.0.0` 或具体网卡 IP），启动网关前必须设置 `MOTORBRIDGE_WS_TOKEN`。
- UI 侧在 WS 握手时需携带 token（`x-motorbridge-token` 或 `Authorization: Bearer ...`）。

### A. Damiao 串口桥（`dm-serial`）

当适配器是 Damiao 串口桥时使用。

Ubuntu：

```bash
motorbridge-gateway -- \
  --bind 127.0.0.1:9002 \
  --vendor damiao --transport dm-serial \
  --serial-port /dev/ttyACM0 --serial-baud 921600 \
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

macOS：

```bash
motorbridge-gateway -- \
  --bind 127.0.0.1:9002 \
  --vendor damiao --transport dm-serial \
  --serial-port /dev/cu.usbmodemXXXX --serial-baud 921600 \
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

如果 macOS 报动态库加载错误，使用包内路径兜底：

```bash
GW="$(python3 -c "import motorbridge, pathlib; print(pathlib.Path(motorbridge.__file__).resolve().parent/'bin'/'ws_gateway')")"
PKG_DIR="$(python3 -c "import motorbridge, pathlib; print(pathlib.Path(motorbridge.__file__).resolve().parent)")"
DYLD_LIBRARY_PATH="$PKG_DIR/lib:${DYLD_LIBRARY_PATH:-}" "$GW" \
  --bind 127.0.0.1:9002 --vendor damiao --transport dm-serial \
  --serial-port /dev/cu.usbmodemXXXX --serial-baud 921600 \
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

Windows（PowerShell）：

```powershell
motorbridge-gateway -- `
  --bind 127.0.0.1:9002 `
  --vendor damiao --transport dm-serial `
  --serial-port COM3 --serial-baud 921600 `
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

### B. PCAN / 标准 CAN 路径（`socketcan`/`pcan`）

当适配器是 PCAN 或标准 CAN 网卡时使用。

Ubuntu（SocketCAN）：

```bash
motorbridge-gateway -- \
  --bind 127.0.0.1:9002 \
  --vendor damiao --transport auto \
  --channel can0 \
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

macOS（PCBUSB 运行时）：

```bash
motorbridge-gateway -- \
  --bind 127.0.0.1:9002 \
  --vendor damiao --transport auto \
  --channel can0 \
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

Windows（PCAN 后端）：

```powershell
motorbridge-gateway -- `
  --bind 127.0.0.1:9002 `
  --vendor damiao --transport auto `
  --channel can0@1000000 `
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

## 3）启动前端

```bash
cd tools/factory_calib_ui_ws
npm run dev
```

打开：`http://127.0.0.1:18110`

UI 连接参数：

- WS URL：`ws://127.0.0.1:9002`
- Channel：与网关启动参数一致（例如 `can0`）

## 4）快速验证

- 通用页：点击 `Scan Damiao`。
- 机械臂页：选 profile，点击 `Prepare 7 Cards`，再点 `Scan All Joints`。

## 5）说明

- Linux SocketCAN 下通道写 `can0`，不要写 `can0@1000000`。
- `dm-serial` 只支持 Damiao。
- 日志里的 `ws disconnected` 常见于浏览器重连，不一定是网关故障；看网关是否仍在监听。

深入排障文档：

- `docs/zh/can_debugging.md`
