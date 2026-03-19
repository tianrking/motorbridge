# 可靠性验证（最小闭环）

本目录用于执行以下可靠性验证：

- 耐久测试（长时间循环控制/读取）
- 异常与超时路径（自动化 + 手工硬件验证）
- 断连恢复（手工硬件验证）
- 跨平台一致性（Linux vs Windows 扫描结果）

## 1）耐久测试

反复执行同一命令并生成 JSON 报告：

```bash
python tools/reliability/reliability_runner.py endurance \
  --command "cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20" \
  --duration-sec 1800 \
  --interval-sec 0.5 \
  --report tools/reliability/reports/windows_endurance_4340p.json
```

通过标准：

- `fail == 0`
- `success_rate == 1.0`

## 2）异常/超时注入

自动化覆盖已纳入 `cargo test --workspace --all-targets`：

- `CoreController` 总线读错误路径
- Damiao 寄存器读取超时
- RobStride 参数读取超时

## 3）断连恢复（硬件在环）

手工步骤：

1. 启动短循环控制（`pos-vel` 或 `vel`）。
2. 拔掉 PCAN-USB（或 Linux 下 `can0` down）。
3. 确认命令出现预期 I/O/超时错误。
4. 重新插回设备 / 恢复总线。
5. 再次执行扫描和控制命令，确认恢复成功。

## 4）跨平台一致性（扫描）

将 Linux 与 Windows 的扫描输出分别保存成日志后对比：

```bash
python tools/reliability/reliability_runner.py compare-scan \
  --left-log tools/reliability/reports/linux_scan.log \
  --right-log tools/reliability/reports/windows_scan.log
```

对比项：

- 各 vendor 的 `hits`
- 各 vendor 发现的 `id` 集合
