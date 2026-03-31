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
- `motor.get_state()`：读取当前状态结构体

## 为什么有时会看到 `state=None`

- `get_state()` 只返回“已经收到的最新反馈”。
- 如果当前这一刻还没收到有效反馈帧，就会是 `None`。
- 新版示例已内置“重试 + poll_feedback_once()”来减少这个现象。
