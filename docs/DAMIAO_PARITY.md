# Damiao Feature Parity

本文件说明当前 Rust 实现与参考 `python-damiao-driver` 的功能对齐状态。

## 1) 已对齐能力

- 控制模式相关
  - MIT / POS_VEL / VEL / FORCE_POS 指令编码与发送
  - 控制模式寄存器（RID=10）检查与切换
- 系统命令
  - enable / disable / clear_error / set_zero_position
- 反馈解码
  - 状态码、位置、速度、力矩、温度
- 寄存器通道
  - 读写命令编码（0x33/0x55）
  - 回包解析（float/u32）
  - `store_parameters`（0xAA）
- 多型号处理
  - 通过型号目录管理 P/V/T 映射范围

## 2) ABI 已覆盖的 Damiao 能力

当前 ABI 已导出完整控制面：

- 四种模式命令发送
- 寄存器读写
- 运维指令（清错/置零/存储/请求反馈/超时设置）
- 状态读取

## 3) 仍需实机确认（建议）

- 具体固件版本下所有寄存器语义一致性
- 边界工况（高速度/高负载）稳定性
- 故障恢复路径（通信丢失、过热等）

## 4) 建议回归清单

1. 上电后 `enable -> ensure_mode(MIT)`
2. 零位保持 + 正弦轨迹
3. 寄存器 10 读写验证
4. `clear_error` / `set_zero_position` 验证
5. 连续运行 10~30 分钟
