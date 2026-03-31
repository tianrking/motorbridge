# 03 模式切换方法

## 本节目标

掌握统一模式切换接口：`motor.ensure_mode(...)`。

## 运行

```bash
python3 bindings/python/get_started/courses/03-mode-switch-method.py
```

## 重点

- 切模式前先 `enable_all()`。
- 切模式后先读一次状态确认。
