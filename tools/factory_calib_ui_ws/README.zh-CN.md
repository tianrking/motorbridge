# factory_calib_ui_ws（React 版）

基于 WebSocket 的工厂上位机（纯前端）。

- 前端技术栈：React + Vite
- 后端控制链路：`integrations/ws_gateway`
- 浏览器仅通过 WS 调用接口：`set_target` / `scan` / `set_id` / `verify` / `enable` / `disable` / `mit` / `pos_vel` / `vel` / `force_pos` / `stop`

## 功能

- 按品牌扫描（`damiao`、`robstride`、`myactuator`、`hightorque`、`hexfellow`）
- Damiao / RobStride 一键扫描按钮
- 扫描到的电机自动加入列表
- 每个电机支持：
  - Enable / Disable / Move / Stop
  - 模式切换 + 目标参数输入
  - Set ID + Verify
- Damiao 扫描参数可视化（`pmax/vmax/tmax`、`detected_by`、`model_guess`）

## 1）启动 ws_gateway

在仓库根目录执行：

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
cargo run -p ws_gateway --release -- \
  --bind 0.0.0.0:9002 \
  --vendor damiao --channel can0 --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

## 2）前端开发模式

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm/tools/factory_calib_ui_ws
npm install
npm run dev
```

打开：`http://127.0.0.1:18110`

如果遇到 `vite: not found`，直接用一键命令：

```bash
npm run dev:ready
```

## 3）构建后本地预览

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm/tools/factory_calib_ui_ws
npm install
npm run build
npm run preview
```

打开：`http://127.0.0.1:18110/`

## 备注

- Linux SocketCAN 通道建议使用 `can0`（不要写成 `can0@1000000`）。
- 当前结构已按“工厂可用”组织，后续可继续扩展滑条控制、轨迹控制、多机批量策略等页面能力。
