# 可靠性验证（最小闭环）

<!-- channel-compat-note -->
## 通道兼容说明（PCAN + slcan + Damiao 串口桥）

- Linux SocketCAN 直接使用网卡名：`can0`、`can1`、`slcan0`。
- 串口类 USB-CAN 需先创建并拉起 `slcan0`：`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`。
- 仅 Damiao 可选串口桥链路：`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`。
- Damiao 串口桥完整接口与命令模板见 `motor_cli/README.zh-CN.md` 第 `3.6` 节（英文见 `motor_cli/README.md`）。
- Linux SocketCAN 下 `--channel` 不要带 `@bitrate`（例如 `can0@1000000` 无效）。
- Windows（PCAN 后端）中，`can0/can1` 映射 `PCAN_USBBUS1/2`，可选 `@bitrate` 后缀。


本目录用于执行以下可靠性验证：

- 耐久测试（长时间循环控制/读取）
- 异常与超时路径（自动化 + 手工硬件验证）
- 断连恢复（手工硬件验证）
- 跨平台一致性（Linux vs Windows 扫描结果）

## 1）耐久测试

反复执行同一命令并生成 JSON 报告：

Windows（PCAN）示例：

```bash
python tools/reliability/reliability_runner.py endurance \
  --command "cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20" \
  --duration-sec 1800 \
  --interval-sec 0.5 \
  --report tools/reliability/reports/windows_endurance_4340p.json
```

Linux（SocketCAN）示例（`can0` 或 `slcan0`）：

```bash
python tools/reliability/reliability_runner.py endurance \
  --command "cargo run -p motor_cli --release -- --vendor damiao --channel slcan0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20" \
  --duration-sec 1800 \
  --interval-sec 0.5 \
  --report tools/reliability/reports/linux_endurance_4340p.json
```

推荐使用模板执行 `slcan0` 专项回归：

```bash
python tools/reliability/reliability_runner.py endurance \
  --template tools/reliability/templates/linux_slcan_endurance_4340p.json
```

可用模板：

- `tools/reliability/templates/linux_slcan_endurance_4340p.json`
- `tools/reliability/templates/linux_slcan_endurance_rs00_vel.json`

通过标准：

- `fail == 0`
- `success_rate == 1.0`

模板中的阈值会自动判定：

- `thresholds.max_fail`
- `thresholds.min_success_rate`

## 2）异常/超时注入

自动化覆盖已纳入 `cargo test --workspace --all-targets`：

- `CoreController` 总线读错误路径
- Damiao 寄存器读取超时
- RobStride 参数读取超时

## 3）断连恢复（硬件在环）

手工步骤：

1. 启动短循环控制（`pos-vel` 或 `vel`）。
2. 拔掉 PCAN-USB（或 Linux 下 `can0` down）。
3. 确认程序会进入自动重试/重连。
4. 重新插回设备 / 恢复总线。
5. 确认循环可恢复；如未恢复，再执行扫描和控制命令验证。

## 4）跨平台一致性（扫描）

将 Linux 与 Windows 的扫描输出分别保存成日志后对比：

```bash
python tools/reliability/reliability_runner.py compare-scan \
  --left-log tools/reliability/reports/linux_scan.log \
  --right-log tools/reliability/reports/windows_scan.log \
  --vendors damiao,robstride \
  --allow-hit-delta 1 \
  --id-mode intersect-nonempty
```

对比项：

- `hits` 可配置容差（`--allow-hit-delta`）
- vendor 子集比较（`--vendors`）
- `id` 比较模式（`--id-mode`）：`exact` / `left-subset` / `right-subset` / `intersect-nonempty`
