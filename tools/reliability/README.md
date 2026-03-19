# Reliability Validation (Minimum Loop)

This folder provides practical checks for:

- endurance (long-run control/read loop)
- error/timeout behavior (automated unit tests + manual HIL checks)
- disconnect/recovery (manual HIL steps)
- cross-platform scan consistency (Linux vs Windows)

## 1) Endurance

Run one command repeatedly and generate a JSON report:

```bash
python tools/reliability/reliability_runner.py endurance \
  --command "cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20" \
  --duration-sec 1800 \
  --interval-sec 0.5 \
  --report tools/reliability/reports/windows_endurance_4340p.json
```

Pass criteria:

- `fail == 0`
- `success_rate == 1.0`

## 2) Error/Timeout Injection

Automated coverage is in `cargo test --workspace --all-targets`:

- bus read error path (`CoreController`)
- Damiao register timeout
- RobStride parameter timeout

## 3) Disconnect / Recovery (HIL)

Manual steps:

1. Start a short control loop (`pos-vel` or `vel`).
2. Unplug PCAN-USB (or down Linux CAN interface).
3. Verify command pauses, then auto-reconnect/retry is attempted.
4. Re-plug / recover bus.
5. Verify loop resumes without restarting the process, or re-run scan/control to confirm recovery.

## 4) Cross-Platform Consistency (Scan)

Save scan stdout logs on Linux and Windows, then compare:

```bash
python tools/reliability/reliability_runner.py compare-scan \
  --left-log tools/reliability/reports/linux_scan.log \
  --right-log tools/reliability/reports/windows_scan.log \
  --vendors damiao,robstride \
  --allow-hit-delta 1 \
  --id-mode intersect-nonempty
```

Comparison checks:

- per-vendor `hits` with optional tolerance (`--allow-hit-delta`)
- vendor subset filtering (`--vendors`)
- configurable `id` matching mode (`--id-mode`):
  - `exact`
  - `left-subset`
  - `right-subset`
  - `intersect-nonempty`
