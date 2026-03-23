# motor_cli (English)

Complete parameter reference for the Rust `motor_cli` binary.

- Crate: `motor_cli`
- Recommended (release package): `./bin/motor_cli [ARGS...]`
- Optional (source build): `./target/release/motor_cli [ARGS...]`

## Release-first Usage

Download and extract the release package (GitHub Releases asset like `motor-cli-vX.Y.Z-linux-x86_64.tar.gz`), then run directly:

```bash
./bin/motor_cli -h
./bin/motor_cli --vendor damiao --mode scan --start-id 1 --end-id 16
```

If you want `motor_cli` as a plain command:

```bash
export PATH="$(pwd)/bin:$PATH"
motor_cli -h
```

## Additional Damiao Command/Register Reference

- Detailed Damiao command + register tuning doc (English): `DAMIAO_API.md`
- 中文版（寄存器/指令详表）: `DAMIAO_API.zh-CN.md`

## Additional RobStride Command/Parameter Reference

- Detailed RobStride command + parameter guide (English): `ROBSTRIDE_API.md`
- 中文版（参数/能力边界详表）: `ROBSTRIDE_API.zh-CN.md`

## Additional MyActuator Command/Mode Reference

- Detailed MyActuator command + mode guide (English): `MYACTUATOR_API.md`
- 中文版（命令/模式详表）: `MYACTUATOR_API.zh-CN.md`

## HighTorque Notes

- Protocol analysis (Chinese): `../docs/zh/hightorque_protocol_analysis.md`
- Current `vendor=hightorque` is a native ht_can v1.5.5 direct-CAN mode, not the official serial-CANboard transport.

## CAN Debugging Entry

- Professional Linux `slcan` + Windows `pcan` troubleshooting: `../docs/en/can_debugging.md`
- 中文调试手册：`../docs/zh/can_debugging.md`

## 1. Argument Parsing Rules

- Only `--key value` style options are parsed.
- A standalone flag (for example `--help`) is treated as value `1`.
- Numeric IDs accept decimal (`20`) and hex (`0x14`).
- Unknown keys are parsed but ignored unless used by code paths.

## 2. Top-Level Arguments (All Vendors)

| Argument | Type | Default | Notes |
|---|---|---|---|
| `--help` | flag | off | Prints CLI help and exits |
| `--vendor` | string | `damiao` | `damiao`, `robstride`, `hightorque`, `myactuator`, `all` |
| `--channel` | string | `can0` | Linux: SocketCAN interface name (`can0`/`slcan0`); Windows (PCAN backend): `can0`/`can1` with optional `@bitrate` suffix (for example `can0@1000000`) |
| `--model` | string | vendor dependent | `4340` for Damiao, `rs-00` for RobStride, `hightorque` for HighTorque, `X8` for MyActuator |
| `--motor-id` | u16 (hex/dec) | `0x01` | Motor CAN ID |
| `--feedback-id` | u16 (hex/dec) | vendor dependent | Damiao `0x11`, RobStride `0xFF`, HighTorque `0x01`, MyActuator `0x241` (for motor-id `1`) |
| `--mode` | string | vendor dependent | Damiao `mit`, RobStride `ping`, HighTorque `read`, MyActuator `status`, `all` -> `scan` |
| `--loop` | u64 | `1` | Control loop cycles |
| `--dt-ms` | u64 | `20` | Loop interval in ms |
| `--ensure-mode` | `0/1` | `1` | Auto-switch mode before control |

### 2.1 Channel Quick Reference (`--channel`)

- Linux SocketCAN:
  - Use interface names directly: `can0`, `can1`, `slcan0`.
  - Configure bitrate at interface setup time (`ip link` / `slcand`), not in `--channel`.
  - `can0@1000000` is invalid on Linux SocketCAN.
- Windows PCAN:
  - `can0` maps to `PCAN_USBBUS1`, `can1` maps to `PCAN_USBBUS2`.
  - Optional bitrate suffix is supported: `can0@1000000`.

## 3. Vendor = `damiao`

### 3.1 Supported Modes

- `scan`
- `enable`
- `disable`
- `mit`
- `pos-vel`
- `vel`
- `force-pos`

### 3.2 Damiao Extra Arguments

| Argument | Type | Default | Used In | Notes |
|---|---|---|---|---|
| `--verify-model` | `0/1` | `1` | non-scan | Verify PMAX/VMAX/TMAX matches `--model` |
| `--verify-timeout-ms` | u64 | `500` | non-scan | Register read timeout for model handshake |
| `--verify-tol` | f32 | `0.2` | non-scan | Model limit tolerance |
| `--start-id` | u16 | `1` | scan | Scan start, must be 1..255 |
| `--end-id` | u16 | `255` | scan | Scan end, must be 1..255 |
| `--set-motor-id` | u16 opt | none | id-set flow | Write ESC_ID (RID 8) |
| `--set-feedback-id` | u16 opt | none | id-set flow | Write MST_ID (RID 7) |
| `--store` | `0/1` | `1` | id-set flow | Persist parameters |
| `--verify-id` | `0/1` | `1` | id-set flow | Re-read RID7/RID8 and verify |

### 3.3 Control Arguments by Mode

| Mode | Arguments | Defaults |
|---|---|---|
| `mit` | `--pos --vel --kp --kd --tau` | `0 0 30 1 0` |
| `pos-vel` | `--pos --vlim` | `0 1.0` |
| `vel` | `--vel` | `0` |
| `force-pos` | `--pos --vlim --ratio` | `0 1.0 0.1` |
| `enable`/`disable` | no extra required | n/a |

### 3.4 Scan Behavior Details

- The scanner is model-agnostic in practice: it internally tries a built-in model-hint list.
- For each candidate ID, it also tries multiple feedback-ID hints: inferred (`id+0x10`), user `--feedback-id`, `0x11`, `0x17`.
- Detection first attempts register reads (RID 21/22/23), then feedback fallback.

### 3.5 Damiao Examples

```bash
# Scan a range
motor_cli \
  --vendor damiao --channel can0 --mode scan --start-id 1 --end-id 16

# MIT control
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --pos 1.57 --vel 2.0 --kp 35 --kd 1.2 --tau 0.3 --loop 120 --dt-ms 20

# Position-velocity control
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode pos-vel --pos 3.14 --vlim 4.0 --loop 120 --dt-ms 20

# Update ID and persist
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --set-motor-id 0x04 --set-feedback-id 0x14 --store 1 --verify-id 1
```

## 4. Vendor = `robstride`

### 4.1 Supported Modes

- `ping`
- `scan`
- `enable`
- `disable`
- `mit`
- `vel`
- `read-param`
- `write-param`

### 4.2 RobStride Extra Arguments

| Argument | Type | Default | Used In | Notes |
|---|---|---|---|---|
| `--start-id` | u16 | `1` | scan | Scan start, 1..255 |
| `--end-id` | u16 | `255` | scan | Scan end, 1..255 |
| `--manual-vel` | f32 | `0.2` | scan fallback | Blind pulse velocity |
| `--manual-ms` | u64 | `200` | scan fallback | Pulse duration per ID |
| `--manual-gap-ms` | u64 | `200` | scan fallback | Gap between IDs |
| `--set-motor-id` | u16 opt | none | id-set flow | Set device ID |
| `--store` | `0/1` | `1` | id-set flow | Save parameters |
| `--param-id` | u16 | required for param modes | read/write param | Parameter ID |
| `--param-value` | typed | required for write | write-param | Parsed by parameter metadata |

### 4.3 Control Arguments by Mode

| Mode | Arguments | Defaults |
|---|---|---|
| `mit` | `--pos --vel --kp --kd --tau` | `0 0 8 0.2 0` |
| `vel` | `--vel` | `0` |
| `enable`/`disable` | no extra required | n/a |

### 4.4 Scan Behavior Details

- Fast pass: ping + query-parameter probe per ID.
- If no hits in full range: fallback to blind velocity pulses for manual movement observation.
- Fallback hit criteria includes state feedback presence.

### 4.5 RobStride Examples

```bash
# Ping
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFF --mode ping

# Scan
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 255

# MIT control
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFF \
  --mode mit --pos 3.14 --vel 0 --kp 8 --kd 0.4 --tau 1.5 --loop 120 --dt-ms 20

# Velocity mode
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFF \
  --mode vel --vel 2.0 --loop 100 --dt-ms 20

# Read parameter
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFF \
  --mode read-param --param-id 0x7005

# Write parameter
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFF \
  --mode write-param --param-id 0x7005 --param-value 2
```

## 5. Vendor = `all`

`vendor=all` currently supports only `--mode scan`.

### 5.1 Additional Arguments for all-scan

| Argument | Default | Notes |
|---|---|---|
| `--damiao-model` | `4340P` | Model hint used when invoking Damiao scan path |
| `--robstride-model` | `rs-00` | Model hint used when invoking RobStride scan path |
| `--hightorque-model` | `hightorque` | Model hint used when invoking HighTorque scan path |
| `--myactuator-model` | `X8` | Model hint used when invoking MyActuator scan path |
| `--start-id` | `1` | Passed to all scans |
| `--end-id` | `255` | Passed to Damiao/RobStride; MyActuator path auto-clamps to `32` |

### 5.2 Example

```bash
motor_cli \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```

## 5.3 Vendor = `hightorque` (native `ht_can` v1.5.5)

- This path uses native HighTorque `ht_can` v1.5.5 direct-CAN protocol.
- It is intended for setups where motors are exposed directly on SocketCAN (`can0` etc.).
- Official Panthera/HighTorque SDK serial chain (`USB serial -> CANboard -> motors`) is separate from this CLI direct-CAN path.
- Supported modes: `scan | read | ping | mit | pos | vel | tqe | pos-vel-tqe | volt | cur | stop | brake | rezero | conf-write | timed-read`.
- Unified unit interface:
  - `--pos` in `rad`
  - `--vel` in `rad/s`
  - `--tau` in `Nm`
  - `--kp`, `--kd` are accepted for MIT signature compatibility but ignored by `ht_can`.
  - Raw debug parameters: `--raw-pos`, `--raw-vel`, `--raw-tqe`.

## 6. Vendor = `myactuator`

### 6.1 Supported Modes

- `scan`
- `enable`
- `disable`
- `stop`
- `set-zero`
- `status`
- `current`
- `vel`
- `pos`
- `version`
- `mode-query`

### 6.2 MyActuator Extra Arguments

| Argument | Type | Default | Used In | Notes |
|---|---|---|---|---|
| `--start-id` | u16 | `1` | scan | Scan start, 1..32 |
| `--end-id` | u16 | `32` | scan | Scan end, 1..32 (input >32 will be clamped) |
| `--current` | f32 | `0.0` | current | Current setpoint in A |
| `--vel` | f32 | `0.0` | vel | Velocity setpoint in rad/s (converted to deg/s internally) |
| `--pos` | f32 | `0.0` | pos | Absolute position in rad (converted to deg internally) |
| `--max-speed` | f32 | `8.726646` | pos | Position move max speed in rad/s (converted internally) |

Status output note:

- `angle` comes from `0x9C` status-2 near-turn angle.
- `mt_angle` comes from `0x92` multi-turn angle and should be used for absolute-position judgement.

### 6.3 MyActuator Examples

```bash
# Scan IDs in MyActuator range
motor_cli \
  --vendor myactuator --channel can0 --mode scan --start-id 1 --end-id 32

# Query status repeatedly
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode status --loop 40 --dt-ms 50

# Velocity control
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode vel --vel 0.5236 --loop 100 --dt-ms 20

# Absolute position control
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode pos --pos 3.1416 --max-speed 5.236 --loop 1

# Set current position as zero (persistent after power-cycle)
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode set-zero --loop 1
```

## 7. Practical Notes

- For Damiao ID updates, prefer keeping `--store 1 --verify-id 1`.
- If scan intermittently misses motors, retry after CAN restart.
- RobStride `send_pos_vel` is not a CLI mode; use `mit` or `vel`.
- MyActuator low-voltage protection returns error code `0x0004` in status-1 (`0x9A`) and blocks motion.
