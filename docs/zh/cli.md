# CLI 使用指南（`motor_cli`）

## 构建

```bash
cargo build -p motor_cli --release
```

## 通用参数

- `--channel`（默认 `can0`）
- `--model`（默认 `4340`）
- `--motor-id`（默认 `0x01`）
- `--feedback-id`（默认 `0x11`）
- `--loop`（默认 `1`）
- `--dt-ms`（默认 `20`）
- `--ensure-mode`（`1/0`，默认 `1`）

型号握手校验（默认开启）：

- `--verify-model 1/0`（默认 `1`）
- `--verify-timeout-ms`（默认 `500`）
- `--verify-tol`（默认 `0.2`）

## 控制模式

### Enable

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode enable --loop 20 --dt-ms 100
```

### Disable

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode disable --loop 20 --dt-ms 100
```

### MIT

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 200 --dt-ms 20
```

### POS_VEL

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode pos-vel --pos 3.10 --vlim 1.50 --loop 300 --dt-ms 20
```

### VEL

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode vel --vel 0.5 --loop 100 --dt-ms 20
```

### FORCE_POS

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode force-pos --pos 0.8 --vlim 2.0 --ratio 0.3 --loop 100 --dt-ms 20
```

## 纯 Rust 改 ID

`motor_cli` 已支持直接改 ID：

- `--set-motor-id <id>` => 写 `rid=8`（`ESC_ID`）
- `--set-feedback-id <id>` => 写 `rid=7`（`MST_ID`）
- `--store 1/0` => 写完是否存储（默认 `1`）
- `--verify-id 1/0` => 重连回读 `rid=8/7`（默认 `1`）

示例（`0x07/0x17` 改为 `0x02/0x12`）：

```bash
cargo run -p motor_cli --release -- \
  --channel can0 --model 4310 --motor-id 0x07 --feedback-id 0x17 \
  --set-motor-id 0x02 --set-feedback-id 0x12 --store 1 --verify-id 1
```

## 扫描 ID（纯 Rust 工作流）

当前 `motor_cli` 还没有独立 `scan` 子命令，可使用 shell 探测循环：

```bash
for i in $(seq 1 16); do
  mid=$(printf "0x%02X" "$i")
  fid=$(printf "0x%02X" $((0x10 + (i & 0x0F))))
  if target/release/motor_cli --channel can0 --model 4340P \
      --motor-id "$mid" --feedback-id "$fid" --mode enable --loop 1 --dt-ms 50 \
      >/tmp/mb_scan.log 2>&1; then
    echo "[hit] motor-id=$mid feedback-id=$fid"
  fi
done
```
