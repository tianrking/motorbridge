# Python Binding 流程化课程（按实操顺序）

这套课程严格按“真实使用流程”组织，每节一对文件：

- `xx-*.py`：可直接运行示例
- `xx-*.md`：本节讲解与调参要点

## 课程顺序

0. `00-enable-and-status`：先使能并查看当前状态
1. `01-scan`：扫描设备，确认 ID
2. `02-register-rw`：读写参数（寄存器）
3. `03-mode-switch-method`：模式切换通用方法
4. `04-mode-mit`：MIT 模式单独使用
5. `05-mode-pos-vel`：POS_VEL 模式单独使用
6. `06-mode-vel`：VEL 模式单独使用
7. `07-mode-force-pos`：FORCE_POS 模式单独使用
8. `08-mode-mixed-switch`：同一程序内混合/切换模式
9. `09-multi-motor`：多电机控制

## 通用约定

- 同一时刻只运行一个发送程序。
- 报 `No buffer space available (os error 105)`：先增大 `DT_MS` 到 `30~50`。
- Linux 设备名通常是 `can0` 或 `slcan0`（USB 串口 CAN 适配器常用 `slcan0`）。
- `Controller` 构造方式：
  - 标准 CAN：`Controller(channel)`
  - Damiao 串口桥：`Controller.from_dm_serial(...)`
  - CAN-FD：`Controller.from_socketcanfd(...)`

## 快速开课

```bash
python3 bindings/python/get_started/courses/00-enable-and-status.py
python3 bindings/python/get_started/courses/01-scan.py
python3 bindings/python/get_started/courses/03-mode-switch-method.py
```

## 课程目录扩展指南（新增/删除品牌、型号、电机）

以 `09-multi-motor.py` 为统一扩展入口：

1. 新增 Damiao 电机
2. 新增 MyActuator 电机
3. 新增 RobStride 电机
4. 删除任意电机
5. 关闭某个品牌

核心原则：

- 同一品牌在同一个 `Controller` 下管理（列表配置）。
- 不同品牌用不同 `Controller`（可以同一条 `can0` 物理总线）。
- 状态查询统一调用风格：`request_feedback + poll_feedback_once + get_state`。

推荐先扫再配：

```bash
python3 bindings/python/get_started/courses/01-scan.py
```

把扫描到的 `id/feedback_id/model` 回填到 `09-multi-motor.py` 的配置列表即可。
