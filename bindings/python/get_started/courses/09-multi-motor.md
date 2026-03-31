# 09 多电机单次查询

## 本节目标

- 同厂商多电机：一个 Controller 下多个 motor handle，一次性查状态。
- 多厂商混合：多个 Controller 并行，一次性查状态。

## 运行

```bash
python3 bindings/python/get_started/courses/09-multi-motor.py
```

## 说明

- Damiao 使用 `DAMIAO_MOTORS` 列表配置，可任意增加数量。
- MyActuator 使用 `USE_MYACTUATOR` + `MYACTUATOR_MOTORS`。
- RobStride 使用 `USE_ROBSTRIDE` + `ROBSTRIDE_MOTORS`。
- 当前统一按 `request_feedback() + poll_feedback_once() + get_state()` 调用。
- `RETRY_ENABLED=True`：开启“查询重试”，可显著减少 `None`。
- `MAX_RETRIES` / `RETRY_DT_MS`：控制重试次数和间隔。

## 如何新增更多 Damiao（重点）

在 `DAMIAO_MOTORS` 里新增一行，例如：

```python
{"name": "dm3", "id": 0x01, "fid": 0x11, "model": "4340P"},
```

你扫描到的示例：
- `probe=0x01` -> `mst_id=0x11`
- `probe=0x04` -> `mst_id=0x14`
- `probe=0x07` -> `mst_id=0x17`

对应可直接写成：

```python
DAMIAO_MOTORS = [
    {"name": "dm1", "id": 0x01, "fid": 0x11, "model": "4340P"},
    {"name": "dm2", "id": 0x04, "fid": 0x14, "model": "4310"},
    {"name": "dm3", "id": 0x07, "fid": 0x17, "model": "4310"},
]
```

## 如何删除某个电机/品牌

- 删除电机：从对应列表删掉那一行。
- 关闭某品牌：把 `USE_MYACTUATOR` 或 `USE_ROBSTRIDE` 设为 `False`。

## 如何新增“全新品牌”支持（代码层）

如果后续要加一个当前脚本里没有的新品牌（例如 `hightorque`）：

1. 新增该品牌配置列表和开关。
2. 新增 `attach_xxx(ctrl, cfgs)`，内部调用对应 `ctrl.add_xxx_motor(...)`。
3. 在 `main()` 里新建该品牌 controller，执行：
   - `ctrl.enable_all()`
   - `states = query_states_with_retry(ctrl, motors)`
   - `append_states(out, names, states)`

这样可以保证扩展后仍保持统一调用范式，客户读起来不混乱。

## RobStride 先扫再填（建议）

```bash
python3 bindings/python/get_started/courses/01-scan.py
```

建议把 `01-scan.py` 中 `VENDOR` 改成 `robstride`，确认扫到的 id 后再回填
`ROBSTRIDE_MOTORS`。
