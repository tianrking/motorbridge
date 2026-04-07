# factory_calib_ui

面向工厂现场的电机 ID 标定上位机（Web 版），当前支持：

- Damiao
- RobStride（扫描 + 改 ID + 校验）
- MyActuator（扫描 + 校验）
- HighTorque（扫描 + 校验）
- Hexfellow（扫描 + 校验）

- 后端：`server.py`（Python 标准库，无三方依赖）
- 标定内核：调用 `motor_cli`（`--mode scan`、`--set-motor-id`、`--set-feedback-id`、`--verify-id`）
- 前端：`index.html`（扫描列表、一键改 ID、回读校验、日志）

## 1. 适用场景

- 批量上线时快速扫描在线电机
- 现场改 `ESC_ID / MST_ID`
- 改完立即回读校验

## 2. 启动

在仓库根目录执行：

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
cargo build -p motor_cli --release
python3 tools/factory_calib_ui/server.py --bind 0.0.0.0 --port 18100
```

打开：

- 本机：`http://127.0.0.1:18100`
- 局域网：`http://<你的IP>:18100`

## 3. 使用流程

1. 先填 `channel`，再按品牌逐行配置（是否扫描、`model`、`start_id`、`end_id` 以及该品牌额外参数）
2. 点 `Scan`，得到在线电机列表
3. 每一行填写 `New ESC_ID / New MST_ID`
4. 点 `Set ID`（默认自动 `store + verify`）
5. 点 `Verify` 可随时回读确认

说明：
- 每个品牌都有独立扫描范围和独立型号配置。
- 不想扫某个品牌时，取消该行 `Scan` 勾选即可。
- Vendor 支持多选，可一次性扫描多个厂商。
- `Set ID` 当前仅支持 Damiao / RobStride。
- RobStride 行的 `New MST_ID` 仅展示，不参与写入（RobStride 改 ID 仅写设备 ID）。

## 4. 注意事项

- 建议一次只连接一台待改地址电机，避免误改。
- Linux SocketCAN 使用 `can0`（不要写 `can0@1000000`）。
- 该 UI 当前聚焦工厂 ID 标定流程，后续可继续扩展更多厂商。

## 5. 常见问题

- 页面无响应：先看右上角 backend 状态，再看日志区命令输出。
- 扫描不到：先检查 `ip -details link show can0`，确认总线和波特率。
- 改 ID 失败：检查 `model` 是否匹配电机型号（如 `4310/4340/4340P`）。


通道排障参考：请查看 docs/zh/can_debugging.md（包含 can0 与 slcan0 的配置说明）。
