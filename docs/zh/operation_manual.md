# 最终用户操作手册（PCAN 主链路 + Damiao 串口桥备用链路）

本文面向实际部署与运维。默认链路是标准 CAN（Linux SocketCAN / Windows PCAN）；Damiao 串口桥（`dm-serial`）仅作为 Damiao 转接板的备用链路。

## 1. 总体原则

- 默认使用标准 CAN 链路：
  - Linux: `can0` / `can1`（SocketCAN，包含 `slcan` 挂载后的网卡）
  - Windows: `can0@1000000`（映射到 PCAN 通道）
- 仅当你使用 Damiao 私有转接板串口桥时，才使用：
  - `--transport dm-serial --serial-port <tty> --serial-baud <baud>`
- `dm-serial` 仅支持 `--vendor damiao`，不用于 RobStride / MyActuator / HighTorque。

## 2. 链路选择

- 场景 A（推荐，通用）：同一总线上有多个厂商或希望统一调试
  - 用标准 CAN（SocketCAN/PCAN）
- 场景 B（Damiao 专板）：Damiao 转接板直连串口，且你需要其私有桥接协议
  - 用 `dm-serial`

## 3. Linux 主链路（SocketCAN）

### 3.1 原生 CAN 网卡（如 PCIe/USB-CAN）

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

### 3.2 SLCAN 适配器（USB 串口转 CAN）

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo slcand -o -c -f -s8 /dev/ttyACM0 can0
sudo ip link set can0 up
ip -details link show can0
```

说明：
- `-s8` 对应 1Mbps。
- 某些 `slcan` 驱动会显示 `bitrate 0`，不等于没生效；以是否能稳定收发为准。

## 4. Linux 备用链路（Damiao 串口桥）

当 Damiao 转接板通过串口直接桥接 CAN 时，使用：

```bash
cargo run -p motor_cli --release -- --vendor damiao \
  --transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600 \
  --model 4310 --mode scan --start-id 1 --end-id 16
```

控制示例：

```bash
cargo run -p motor_cli --release -- --vendor damiao \
  --transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600 \
  --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --verify-model 0 --ensure-mode 0 \
  --pos 0.5 --vel 0 --kp 20 --kd 1 --tau 0 --loop 80 --dt-ms 20
```

## 5. Windows 主链路（PCAN）

先安装 PEAK 驱动和 `PCANBasic.dll`，然后使用：

```bash
cargo run -p motor_cli --release -- --vendor damiao \
  --channel can0@1000000 --model 4310 --mode scan --start-id 1 --end-id 16
```

说明：
- `can0` 映射 `PCAN_USBBUS1`，`can1` 映射 `PCAN_USBBUS2`。
- Windows 也可使用 Damiao 串口桥（`dm-serial`），串口名常见为 `COM3`、`COM4` 等。

## 6. 多厂商统一控制建议

- 你要统一控制 Damiao + 其他厂商时：
  - 必须走标准 CAN（SocketCAN/PCAN）
- 你使用 Damiao 串口桥时：
  - 只保证 Damiao 协议路径可用
  - 文档和 CLI 会明确提示其为 Damiao-only

## 7. 常见问题与排障

### 7.1 `if_nametoindex failed for can0: No such device`

原因：`can0` 未创建或已掉线。  
处理：重新执行第 3 章起链流程，确认 `ip -details link show can0` 可见。

### 7.2 `model handshake failed reading PMAX(rid=21)`

常见原因：
- 电机 ID / 反馈 ID 不匹配
- 链路不通（接线、终端电阻、电源）
- 用错链路（本应走 Damiao 串口桥却走了 `can0`，或反之）

建议顺序：
1. 先 `--mode scan` 确认在线 ID。
2. 再用命中的 `motor-id/feedback-id` 做 `--loop 1` 小步验证。
3. 仍失败时抓包（`candump can0`）或切换到 `dm-serial` 交叉验证。

### 7.3 两块 Damiao 转接板跑一段时间后失联

优先排查：
- USB 供电/线材/Hub 稳定性
- 串口设备号漂移（`/dev/ttyACM*` 变化）
- 多适配器竞争（重复启动 `slcand`、接口名冲突）

建议：
- 使用固定规则（udev）绑定设备名
- 每路独立接口名与启动脚本
- 持续监控 `dmesg` / `journalctl -k`

## 8. 推荐运行基线

- 生产默认：
  - Linux：SocketCAN（可含 `slcan`）
  - Windows：PCAN
- Damiao 串口桥：
  - 作为备用链路启用
  - 在文档与命令中显式加 `--transport dm-serial`

## 9. 相关文档

- CLI 参考：[cli.md](cli.md)
- ABI 参考：[abi.md](abi.md)
- CAN 调试：[can_debugging.md](can_debugging.md)
- 顶层说明：[`README.zh-CN.md`](../../README.zh-CN.md)
