# 00 使能与状态查询

## 本节目标

先确认“链路 + 电机句柄 + 反馈”三件事都通了。

## 运行

```bash
python3 bindings/python/get_started/courses/00-enable-and-status.py
```

## 关键接口

- `ctrl.enable_all()`：使能控制输出
- `motor.request_feedback()`：主动请求一次反馈
- `ctrl.poll_feedback_once()`：手动推进一次反馈接收/解析（兼容旧版本写法）
- `motor.get_state()`：读取当前状态结构体

## 版本说明（重要）

- `<= v0.1.6`：建议保留 `poll_feedback_once()`，否则更容易出现 `state=None`。
- `v0.1.7+`：默认已启用后台轮询，通常不再需要手动 `poll_feedback_once()`。
- 为了兼容旧代码，本课程示例仍保留该调用。

## 为什么有时会看到 `state=None`

- `get_state()` 只返回“已经收到的最新反馈”。
- 如果当前这一刻还没收到有效反馈帧，就会是 `None`。
- 示例已内置“重试 + poll_feedback_once()”来减少这个现象（新版本里该 poll 可选）。
