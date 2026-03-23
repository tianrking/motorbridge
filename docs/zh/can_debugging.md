# CAN 调试指南（Linux `slcan` + Windows `pcan`）

本文档是本项目通道配置与链路排障的统一手册。

## 1. 范围与后端映射

- Linux 后端：SocketCAN（`can0`、`can1`、`slcan0` 等）
- Windows 后端：PEAK PCAN（依赖 `PCANBasic.dll`，`can0/can1` 映射到 `PCAN_USBBUS1/2`）

规则：

- Linux：波特率在网卡初始化时设置，不要写进 `--channel`。
- Windows PCAN：`--channel` 支持可选 `@bitrate` 后缀（如 `can0@1000000`）。

## 2. Linux `slcan` 初始化与验收

### 2.1 拉起 `slcan0`

```bash
sudo pkill slcand 2>/dev/null || true
sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0
sudo ip link set slcan0 up
ip -details link show slcan0
```

期望结果：

- 能看到 `slcan0` 网卡
- 状态为 `UP`（或 `UNKNOWN`）

### 2.2 链路最小自检

终端 A：

```bash
candump slcan0
```

终端 B（发测试帧）：

```bash
cansend slcan0 001#1122334455667788
```

如果 `candump` 无帧，优先检查接线、终端电阻、以及电机端波特率是否一致。

### 2.3 在 `slcan0` 上跑 `motor_cli`

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel slcan0 --mode scan --start-id 1 --end-id 16
```

## 3. Linux 原生 SocketCAN（`can0`）快检

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

重点观察计数器：

- `RX errors`、`TX errors`、`bus-off`、`re-started`

若 `bus-off` 持续增长，先处理物理层：终端电阻、地线参考、电机波特率一致性。

## 4. Windows PCAN 初始化与验收

### 4.1 前置条件

- 已安装 PEAK 驱动
- 已安装 PCAN-Basic 运行时（`PCANBasic.dll` 可加载）
- 在 PEAK 工具中可见对应 USB 通道

### 4.2 本项目通道约定

- `can0` -> `PCAN_USBBUS1`
- `can1` -> `PCAN_USBBUS2`
- 可选波特率后缀：`can0@1000000`

### 4.3 验证命令

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16
```

若报 `load PCANBasic.dll failed`，先解决运行时/DLL 搜索路径问题。

## 5. 报错到动作（Error-to-Action）

### Linux SocketCAN 路径

- `if_nametoindex failed ...`：
  - 通道名错误，或网卡未创建/未拉起
  - 动作：`ip link show`，重新创建 `slcan0` 或拉起 `can0`
- `socketcan write failed` / `socketcan read failed` 且提示 `interface is down`：
  - 动作：`ip -details link show <ifname>`，再执行 `ip link set <ifname> up`
- `... unavailable` / `interface not found`：
  - 动作：检查 USB-CAN 连接与网卡命名

### Windows PCAN 路径

- `load PCANBasic.dll failed`：
  - 动作：安装 PCAN-Basic，重开终端/IDE 使 DLL 可被加载
- `PCAN initialize failed: status=...`：
  - 动作：核对 `can0/can1` 映射、`@bitrate`、适配器占用状态
- 持续重连失败：
  - 动作：检查线缆、终端电阻、供电和 PEAK 通道占用

## 6. 跨平台最小验收清单

- Linux `can0`：扫描能返回预期电机 ID
- Linux `slcan0`：按 `slcand` 初始化后扫描同样可通
- Windows `can0@1000000`：扫描能成功
- 各环境至少执行 1 条控制命令（`mit` 或 `pos-vel`）成功

四项都通过，即可判定 `pcan + slcan` 支持已对齐。

