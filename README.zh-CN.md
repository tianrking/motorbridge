# motorbridge

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10--3.14-blue.svg)](https://www.python.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platforms](https://img.shields.io/badge/Platforms-Linux%20%7C%20Windows%20%7C%20macOS-6f42c1.svg)](README.zh-CN.md#å‘å¸ƒä¸Žå®‰è£…æ€»è§ˆå®Œæ•´çŸ©é˜µ)
[![GitHub Release](https://img.shields.io/github/v/release/tianrking/motorbridge)](https://github.com/tianrking/motorbridge/releases)

è¿™æ˜¯ä¸€ä¸ªç»Ÿä¸€çš„ CAN ç”µæœºæŽ§åˆ¶æ ˆï¼ŒåŒ…å« vendor-agnostic Rust coreã€ç¨³å®š C ABIï¼Œä»¥åŠ Python/C++ bindingsã€‚

> English version: [README.md](README.md)

## ä¼ è¾“é“¾è·¯æ ‡è¯†

- `[STD-CAN]`ï¼šæ ‡å‡† CAN è·¯å¾„ï¼ˆ`socketcan` / `pcan`ï¼‰
- `[CAN-FD]`ï¼šç‹¬ç«‹ CAN-FD è·¯å¾„ï¼ˆ`socketcanfd`ï¼‰
- `[DM-SERIAL]`ï¼šDamiao ä¸²å£æ¡¥è·¯å¾„ï¼ˆ`dm-serial`ï¼‰

å½“å‰çŠ¶æ€ï¼š
- `[CAN-FD]` å·²å®Œæˆç‹¬ç«‹é“¾è·¯æŽ¥å…¥ã€‚
- ä»“åº“å†…å°šæœªå£°æ˜Žâ€œæŸä¸ªç”µæœºåž‹å·å·²å®Œæˆ CAN-FD é‡äº§çº§éªŒè¯çŸ©é˜µâ€ã€‚

## å½“å‰æ”¯æŒçš„åŽ‚å•†

- Damiao:
  - åž‹å·: `3507`, `4310`, `4310P`, `4340`, `4340P`, `6006`, `8006`, `8009`, `10010L`, `10010`, `H3510`, `G6215`, `H6220`, `JH11`, `6248P`
  - æ¨¡å¼: `scan`, `MIT`, `POS_VEL`, `VEL`, `FORCE_POS`
- RobStride:
  - åž‹å·: `rs-00`, `rs-01`, `rs-02`, `rs-03`, `rs-04`, `rs-05`, `rs-06`
  - æ¨¡å¼: `scan`, `ping`, `MIT`, `POS_VEL`, `VEL`, å‚æ•°è¯»å†™
  - è¯´æ˜Ž: åŠ›çŸ©/ç”µæµå½“å‰ä»…æ”¯æŒå‚æ•°çº§å†™å…¥ï¼ˆå¦‚ `iq_ref`/é™å¹…å‚æ•°ï¼‰ï¼Œå°šæœªå¼€æ”¾ä¸ºç»Ÿä¸€é«˜å±‚æ¨¡å¼
- MyActuator:
  - åž‹å·: `X8`ï¼ˆè¿è¡Œæ—¶å­—ç¬¦ä¸²ï¼Œåè®®æŒ‰ ID é€šä¿¡ï¼‰
  - æ¨¡å¼: `scan`, `enable`, `disable`, `stop`, `set-zero`, `status`, `current`, `vel`, `pos`, `version`, `mode-query`
- HighTorque:
  - åž‹å·: `hightorque`ï¼ˆè¿è¡Œæ—¶å­—ç¬¦ä¸²ï¼ŒåŽŸç”Ÿ `ht_can v1.5.5`ï¼‰
  - æ¨¡å¼: `scan`, `read`, `mit`, `pos-vel`, `vel`, `stop`, `brake`, `rezero`
- Hexfellow:
  - åž‹å·: `hexfellow`ï¼ˆè¿è¡Œæ—¶å­—ç¬¦ä¸²ï¼ŒCANopen é…ç½®ï¼‰
  - æ¨¡å¼: `scan`, `status`, `enable`, `disable`, `pos-vel`, `mit`ï¼ˆé€šè¿‡ `socketcanfd`ï¼‰

## 更新说明（2026-04）：Damiao / RobStride 能力收敛

- Damiao：`scan / enable / disable / MIT / POS_VEL / VEL / FORCE_POS / set-id / set-zero` 均已纳入生产基线。
- RobStride：`scan / ping / enable / disable / MIT / POS_VEL / VEL / 参数读写 / set-id / zero` 已可用。
- RobStride 默认 host/feedback 路径为 `0xFD`（内部回退探测 `0xFF/0xFE`）。
- RobStride `pos-vel` 的 `--vel/--kd/--tau` 属于无效参数：CLI 仅 warning，不会中断。

## æž¶æž„

### åˆ†å±‚è¿è¡Œæ—¶è§†å›¾

```mermaid
flowchart TB
  APP["ä¸Šå±‚åº”ç”¨ï¼ˆRust/C/C++/Python/ROS2/WSï¼‰"] --> SURFACE["CLI / ABI / SDK / Integrations"]
  SURFACE --> CORE["motor_coreï¼ˆCoreController / bus / model / traitsï¼‰"]
  CORE --> RX["åŽå°æŽ¥æ”¶çº¿ç¨‹ï¼ˆé»˜è®¤ï¼‰"]
  CORE --> MANUAL["poll_feedback_once()ï¼ˆå…¼å®¹æ‰‹åŠ¨è°ƒç”¨ï¼‰"]
  RX --> CACHE["æ¯ç”µæœºæœ€æ–°çŠ¶æ€ç¼“å­˜"]
  MANUAL --> CACHE
  CORE --> DAMIAO["motor_vendors/damiao"]
  CORE --> ROBSTRIDE["motor_vendors/robstride"]
  CORE --> MYACT["motor_vendors/myactuator"]
  CORE --> HIGHTORQUE["motor_vendors/hightorque"]
  CORE --> HEXFELLOW["motor_vendors/hexfellow"]
  CORE --> TEMPLATE["motor_vendors/templateï¼ˆæŽ¥å…¥æ¨¡æ¿ï¼‰"]
  DAMIAO --> CAN["CAN æ€»çº¿åŽç«¯"]
  ROBSTRIDE --> CAN
  MYACT --> CAN
  HIGHTORQUE --> CAN
  HEXFELLOW --> CAN
  CAN --> LNX["Linuxï¼šSocketCAN"]
  CAN --> WIN["Windowsï¼ˆå®žéªŒï¼‰ï¼šPEAK PCAN"]
  CAN --> HW["çœŸå®žç”µæœºç¡¬ä»¶"]
```

### å·¥ä½œåŒºæ‹“æ‰‘ï¼ˆæœ€æ–°ç‰ˆï¼‰

```mermaid
flowchart LR
  ROOT["motorbridge workspace"] --> CORE["motor_core"]
  ROOT --> VENDORS["motor_vendors/*"]
  ROOT --> CLI["motor_cli"]
  ROOT --> ABI["motor_abi"]
  ROOT --> TOOLS["tools/factory_calib_ui"]
  ROOT --> INTS["integrations/*"]
  ROOT --> BIND["bindings/*"]
  VENDORS --> VD["damiao"]
  VENDORS --> VH["hexfellow"]
  VENDORS --> VHT["hightorque"]
  VENDORS --> VR["robstride"]
  VENDORS --> VM["myactuator"]
  VENDORS --> VT["template"]
  INTS --> ROS["ros2_bridge"]
  INTS --> WS["ws_gateway"]
  BIND --> PY["python"]
  BIND --> CPP["cpp"]
```

### Python Binding æŽ¥å£è§†å›¾ï¼ˆv0.1.7+ï¼‰

```mermaid
flowchart TB
  PYAPP["Python åº”ç”¨"] --> CTL["Controller(...) / from_dm_serial(...) / from_socketcanfd(...)"]
  CTL --> ADD["add_damiao / add_robstride / add_myactuator / add_hightorque / add_hexfellow"]
  ADD --> MOTOR["MotorHandle"]
  MOTOR --> CTRL1["send_mit / send_pos_vel / send_vel / send_force_pos"]
  MOTOR --> CTRL2["ensure_mode / enable / disable / set_zero / stop / clear_error"]
  MOTOR --> FB1["request_feedback()"]
  CTL --> FB2["poll_feedback_once()ï¼ˆå‘åŽå…¼å®¹ï¼‰"]
  FB1 --> STATE["get_state() è¯»å–æœ€æ–°ç¼“å­˜çŠ¶æ€"]
  FB2 --> STATE
```

- [`motor_core`](motor_core): ä¸ŽåŽ‚å•†æ— å…³çš„æŽ§åˆ¶å™¨ã€è·¯ç”±ã€CAN æ€»çº¿å±‚ï¼ˆLinux SocketCAN / Windows å®žéªŒæ€§ PCANï¼‰
- [`motor_vendors/damiao`](motor_vendors/damiao): Damiao åè®® / åž‹å· / å¯„å­˜å™¨
- [`motor_vendors/hexfellow`](motor_vendors/hexfellow): Hexfellow CANopen-over-CAN-FD å®žçŽ°
- [`motor_vendors/hightorque`](motor_vendors/hightorque): HighTorque åŽŸç”Ÿ ht_can åè®®å®žçŽ°
- [`motor_vendors/robstride`](motor_vendors/robstride): RobStride æ‰©å±• CAN åè®® / åž‹å· / å‚æ•°
- [`motor_vendors/myactuator`](motor_vendors/myactuator): MyActuator CAN åè®®å®žçŽ°
- [`motor_cli`](motor_cli): ç»Ÿä¸€ Rust CLI
  - å…¨å‚æ•°è‹±æ–‡æ–‡æ¡£: [`motor_cli/README.md`](motor_cli/README.md)
  - å…¨å‚æ•°ä¸­æ–‡æ–‡æ¡£: [`motor_cli/README.zh-CN.md`](motor_cli/README.zh-CN.md)
  - Damiao æŒ‡ä»¤/å¯„å­˜å™¨æ–‡æ¡£: [`motor_cli/DAMIAO_API.md`](motor_cli/DAMIAO_API.md), [`motor_cli/DAMIAO_API.zh-CN.md`](motor_cli/DAMIAO_API.zh-CN.md)
  - RobStride æŒ‡ä»¤/å‚æ•°æ–‡æ¡£: [`motor_cli/ROBSTRIDE_API.md`](motor_cli/ROBSTRIDE_API.md), [`motor_cli/ROBSTRIDE_API.zh-CN.md`](motor_cli/ROBSTRIDE_API.zh-CN.md)
  - MyActuator æŒ‡ä»¤/æ¨¡å¼æ–‡æ¡£: [`motor_cli/MYACTUATOR_API.md`](motor_cli/MYACTUATOR_API.md), [`motor_cli/MYACTUATOR_API.zh-CN.md`](motor_cli/MYACTUATOR_API.zh-CN.md)
- [`motor_abi`](motor_abi): ç¨³å®š C ABI
- [`bindings/python`](bindings/python): Python SDK + `motorbridge-cli`
- [`bindings/cpp`](bindings/cpp): C++ RAII wrapper

## å¿«é€Ÿå¼€å§‹

æž„å»º:

```bash
cargo build
```

æ‹‰èµ· CAN:

```bash
sudo ip link set can0 down 2>/dev/null || true
sudo ip link set can0 type can bitrate 1000000 restart-ms 100
sudo ip link set can0 up
ip -details link show can0
```

Linux ä¸‹å¿«é€Ÿé‡å¯ CANï¼š

```bash
# é»˜è®¤ï¼šcan0 / 1Mbps / restart-ms=100 / loopback å…³é—­
IF=can0; BITRATE=1000000; RESTART_MS=100; LOOPBACK=off
sudo ip link set "$IF" down 2>/dev/null || true
if [ "$LOOPBACK" = "on" ]; then
  sudo ip link set "$IF" type can bitrate "$BITRATE" restart-ms "$RESTART_MS" loopback on
else
  sudo ip link set "$IF" type can bitrate "$BITRATE" restart-ms "$RESTART_MS" loopback off
fi
sudo ip link set "$IF" up
ip -details link show "$IF"
```

Damiao CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \
  --mode mit --pos 0 --vel 0 --kp 20 --kd 1 --tau 0 --loop 50 --dt-ms 20
```
`[STD-CAN]`

Hexfellow CLIï¼š

```bash
cargo run -p motor_cli --release -- \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --model hexfellow --motor-id 1 --feedback-id 0 \
  --mode status
```
`[CAN-FD]`

RobStride CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

RobStride CLI è¯»å‚æ•°:

```bash
cargo run -p motor_cli --release -- \
  --vendor robstride --channel can0 --model rs-00 --motor-id 127 \
  --mode read-param --param-id 0x7019
```

HighTorque CLIï¼ˆåŽŸç”Ÿ ht_can v1.5.5ï¼‰:

```bash
cargo run -p motor_cli --release -- \
  --vendor hightorque --channel can0 --model hightorque --motor-id 1 \
  --mode read
```

MyActuator CLI:

```bash
cargo run -p motor_cli --release -- \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode status --loop 20 --dt-ms 50
```

ç»Ÿä¸€å…¨å“ç‰Œæ‰«æ:

```bash
cargo run -p motor_cli --release -- \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```

## Windows å®žéªŒæ”¯æŒï¼ˆPCAN-USBï¼‰

é¡¹ç›®ä¸»çº¿ä»ä»¥ Linux ä¸ºä¸»ã€‚Windows æ”¯æŒä¸ºå®žéªŒæ€§èƒ½åŠ›ï¼Œå½“å‰é€šè¿‡ PEAK PCAN åŽç«¯å®žçŽ°ã€‚

- åœ¨ Windows å®‰è£… PEAK é©±åŠ¨ä¸Ž PCAN-Basic è¿è¡Œæ—¶ï¼ˆ`PCANBasic.dll`ï¼‰ã€‚
- é€šé“æ˜ å°„ï¼š
  - `can0` -> `PCAN_USBBUS1`
  - `can1` -> `PCAN_USBBUS2`
- å¯é€‰æ³¢ç‰¹çŽ‡åŽç¼€ï¼š`@<bitrate>`ï¼Œä¾‹å¦‚ `can0@1000000`ã€‚

Windows éªŒè¯å‘½ä»¤ï¼š

```bash
# æ‰«æ Damiao ç”µæœº ID
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode scan --start-id 1 --end-id 16

# 1 å·ç”µæœºï¼ˆ4340Pï¼‰è½¬åˆ° +pi å¼§åº¦ï¼ˆçº¦ 180 åº¦ï¼‰
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4340P --motor-id 0x01 --feedback-id 0x11 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20

# 7 å·ç”µæœºï¼ˆ4310ï¼‰è½¬åˆ° +pi å¼§åº¦ï¼ˆçº¦ 180 åº¦ï¼‰
cargo run -p motor_cli --release -- --vendor damiao --channel can0@1000000 --model 4310 --motor-id 0x07 --feedback-id 0x17 --mode pos-vel --pos 3.1416 --vlim 2.0 --loop 1 --dt-ms 20
```

## Linux USB-CANï¼ˆ`slcan`ï¼‰é€ŸæŸ¥

Linux ä¸‹ç›´æŽ¥ä½¿ç”¨ SocketCAN ç½‘å¡åï¼ˆä¾‹å¦‚ `can0`ã€`slcan0`ï¼‰ã€‚
ä¸è¦åœ¨ Linux çš„é€šé“åé‡ŒåŠ æ³¢ç‰¹çŽ‡åŽç¼€ï¼ˆä¾‹å¦‚ `can0@1000000` åœ¨ Linux SocketCAN ä¸‹æ— æ•ˆï¼‰ã€‚

æŠŠ `slcan` é€‚é…å™¨æŒ‚æˆ `slcan0`ï¼š

```bash
sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0
sudo ip link set slcan0 up
ip -details link show slcan0
```

ä¹‹åŽåœ¨ CLI é‡Œç›´æŽ¥ä½¿ç”¨ `slcan0`ï¼š

```bash
cargo run -p motor_cli --release -- --vendor damiao --channel slcan0 --mode scan --start-id 1 --end-id 255
```

## Damiao ç‹¬ç«‹ CAN-FD ä¼ è¾“ï¼ˆ`socketcanfd`ï¼‰

å½“ä½ å¸Œæœ›å¢žåŠ ä¸€æ¡ä¸Žç»å…¸ CANã€`dm-serial` å¹¶å­˜çš„ Linux CAN-FD é“¾è·¯æ—¶ï¼Œå¯ä½¿ç”¨è¯¥ transportã€‚

```bash
# å…ˆæŠŠ can0 é…æˆ FD æ¨¡å¼
scripts/canfd_restart.sh can0

# Damiao èµ°ç‹¬ç«‹ socketcanfd é“¾è·¯
cargo run -p motor_cli --release -- --vendor damiao \
  --transport socketcanfd --channel can0 \
  --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --verify-model 0 --ensure-mode 0 \
  --pos 0.5 --vel 0 --kp 20 --kd 1 --tau 0 --loop 80 --dt-ms 20
```
`[CAN-FD]`ï¼ˆå·²æŽ¥å…¥é“¾è·¯ï¼Œç”µæœºéªŒè¯çŸ©é˜µå¾…è¡¥ï¼‰

## Damiao ä¸²å£æ¡¥é€ŸæŸ¥ï¼ˆ`dm-serial`ï¼‰

å½“ä½ çš„ Damiao è½¬æŽ¥æ¿æä¾›ä¸²å£æ¡¥ï¼ˆä¾‹å¦‚ `/dev/ttyACM1`ï¼‰ä¸”å¸Œæœ›èµ°è¿™æ¡ç§æœ‰é“¾è·¯æ—¶ï¼Œå¯ä½¿ç”¨ï¼š

```bash
# Damiao ä¸²å£æ¡¥æ‰«æ
cargo run -p motor_cli --release -- --vendor damiao \
  --transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600 \
  --model 4310 --mode scan --start-id 1 --end-id 16

# Damiao ä¸²å£æ¡¥ MIT æŽ§åˆ¶
cargo run -p motor_cli --release -- --vendor damiao \
  --transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600 \
  --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --verify-model 0 --ensure-mode 0 \
  --pos 0.5 --vel 0 --kp 20 --kd 1 --tau 0 --loop 80 --dt-ms 20
```
`[DM-SERIAL]`

## CAN ä¸“ä¸šè°ƒè¯•æ‰‹å†Œ

å¦‚éœ€ç³»ç»ŸåŒ–æŽ’æŸ¥ Linux `slcan` ä¸Ž Windows `pcan`ï¼Œè¯·ç›´æŽ¥ä½¿ç”¨ï¼š

- [`docs/zh/can_debugging.md`](docs/zh/can_debugging.md)
- [`docs/en/can_debugging.md`](docs/en/can_debugging.md)

æœ€ç»ˆç”¨æˆ·å®Œæ•´é“¾è·¯æ“ä½œï¼ˆé»˜è®¤ PCAN/SocketCANï¼ŒDamiao ä¸²å£æ¡¥å¤‡ç”¨ï¼‰è¯·çœ‹ï¼š

- [`docs/zh/operation_manual.md`](docs/zh/operation_manual.md)

ç»“æžœè§£è¯»ï¼š

- `vendor=damiao id=<n>`ï¼šå‘çŽ°ä¸€ä¸ª Damiao ç”µæœºï¼Œç”µæœº ID ä¸º `<n>`ã€‚
- `vendor=robstride id=<n> responder_id=<m>`ï¼šå‘çŽ°ä¸€ä¸ª RobStride ç”µæœºå¹¶è¿”å›žå“åº” IDã€‚
- `vendor=hightorque ... [hit] id=<n> ...`ï¼šé€šè¿‡åŽŸç”Ÿ ht_can v1.5.5 å‘çŽ°ä¸€ä¸ª HighTorque ç”µæœºã€‚
- `vendor=myactuator id=<n>`ï¼šå‘çŽ°ä¸€ä¸ª MyActuator ç”µæœºå¹¶è¿”å›žç‰ˆæœ¬å“åº”ã€‚
- æ¯æ®µæ‰«æç»“å°¾çš„ `hits=<k>` è¡¨ç¤ºè¯¥åŽ‚å•†å‘½ä¸­çš„åœ¨çº¿è®¾å¤‡æ•°é‡ã€‚

## ABI ä¸Žç»‘å®š

- C ABI:
  - `motor_controller_new_socketcan(channel)`
  - `motor_controller_new_dm_serial(serial_port, baud)`ï¼ˆä»… Damiao ä¸²å£æ¡¥ï¼›è·¨å¹³å°ï¼Œå¯ç”¨ `/dev/ttyACM0` æˆ– `COM3`ï¼‰
  - Damiao: `motor_controller_add_damiao_motor(...)`
  - Hexfellow: `motor_controller_add_hexfellow_motor(...)`ï¼ˆé€šè¿‡ `socketcanfd` èµ° CAN-FDï¼‰
  - RobStride: `motor_controller_add_robstride_motor(...)`
  - MyActuator: `motor_controller_add_myactuator_motor(...)`
  - HighTorque: `motor_controller_add_hightorque_motor(...)`
- Python:
  - `Controller(channel="can0")`
  - `Controller.from_dm_serial("/dev/ttyACM0", 921600)`ï¼ˆä»… Damiaoï¼‰
  - `Controller.add_damiao_motor(...)`
  - `Controller.add_hexfellow_motor(...)`
  - `Controller.add_robstride_motor(...)`
  - `Controller.add_myactuator_motor(...)`
  - `Controller.add_hightorque_motor(...)`
- C++:
  - `Controller("can0")`
  - `Controller::from_dm_serial("/dev/ttyACM0", 921600)`ï¼ˆä»… Damiaoï¼‰
  - `Controller::add_damiao_motor(...)`
  - `Controller::add_hexfellow_motor(...)`
  - `Controller::add_robstride_motor(...)`
  - `Controller::add_myactuator_motor(...)`
  - `Controller::add_hightorque_motor(...)`

ABI/ç»‘å®šä¸­çš„ç»Ÿä¸€æ¨¡å¼ IDï¼ˆ`ensure_mode`ï¼‰ï¼š

- `1 = MIT`
- `2 = POS_VEL`
- `3 = VEL`
- `4 = FORCE_POS`

ç»Ÿä¸€æŽ§åˆ¶å•ä½ï¼š

- ä½ç½®ï¼š`rad`
- é€Ÿåº¦ï¼š`rad/s`
- åŠ›çŸ©ï¼š`Nm`

å„åŽ‚å•†åè®®åŽŸç”Ÿæ¨¡å¼åæ˜ å°„ä¸Žä¸æ”¯æŒé¡¹è¯¦è§ï¼š

- [`docs/en/abi.md`](docs/en/abi.md)
- [`docs/zh/abi.md`](docs/zh/abi.md)

RobStride ä¸“å±ž ABI / binding èƒ½åŠ›åŒ…æ‹¬:

- `robstride_ping`
- `robstride_get_param_*`
- `robstride_write_param_*`

## ç¤ºä¾‹å…¥å£

- è·¨è¯­è¨€ç´¢å¼•: `examples/README.md`
- C ABI ç¤ºä¾‹: `examples/c/c_abi_demo.c`
- C++ ABI ç¤ºä¾‹: `examples/cpp/cpp_abi_demo.cpp`
- Python ctypes ç¤ºä¾‹: `examples/python/python_ctypes_demo.py`
- Python SDK æ–‡æ¡£: `bindings/python/README.md`
- C++ binding æ–‡æ¡£: `bindings/cpp/README.md`

## å‘å¸ƒä¸Žå®‰è£…æ€»è§ˆï¼ˆå®Œæ•´çŸ©é˜µï¼‰

### A) GitHub Releasesï¼ˆäºŒè¿›åˆ¶èµ„äº§ï¼‰

| èµ„äº§ | å®‰è£… / ä½¿ç”¨æ–¹å¼ | å¹³å° | é€‚ç”¨äººç¾¤ | åŒ…å«èƒ½åŠ› |
|---|---|---|---|---|
| `motorbridge-abi-<tag>-linux-x86_64.deb` | `sudo apt install ./motorbridge-abi-<tag>-linux-x86_64.deb` | Linux x86_64 | C/C++ ç”¨æˆ·ï¼ˆUbuntu/Debianï¼‰ | `libmotor_abi` + å¤´æ–‡ä»¶ + CMake é…ç½® |
| `motorbridge-abi-<tag>-linux-*.tar.gz` | è§£åŽ‹åŽæ‰‹åŠ¨é“¾æŽ¥ | Linux x86_64/aarch64 | C/C++ ç”¨æˆ·ï¼ˆéž deb çŽ¯å¢ƒï¼‰ | ä¸Ž `.deb` åŒç­‰ ABI å†…å®¹ |
| `motorbridge-abi-<tag>-windows-x86_64.zip` | è§£åŽ‹åŽé“¾æŽ¥/åŠ è½½ | Windows x86_64 | C/C++ ç”¨æˆ· | `motor_abi.dll/.lib` + å¤´æ–‡ä»¶ + CMake é…ç½® |
| `motor-cli-<tag>-<platform>.tar.gz/.zip` | ç›´æŽ¥è¿è¡Œ `bin/motor_cli` | Linux/Windows | çŽ°åœºè°ƒè¯•/å·¥åŽ‚å·¥å…· | ç»Ÿä¸€ CLI èƒ½åŠ›ï¼ˆæ‰«æã€æŽ§åˆ¶ã€æ”¹ ID ç­‰ï¼‰ |
| `motorbridge-*.whl`, `motorbridge-*.tar.gz` | `pip install ./...` | å–å†³äºŽ wheel tag | ç¦»çº¿ Python å®‰è£… | Python SDK + `motorbridge-cli` |

### B) PyPI / TestPyPIï¼ˆPython åŒ…åˆ†å‘ï¼‰

| é€šé“ | å‘å¸ƒè§¦å‘æ–¹å¼ | Python ç‰ˆæœ¬ | å¹³å°çŸ©é˜µ | åŒ…ç±»åž‹ |
|---|---|---|---|---|
| TestPyPI | `Actions -> Python Publish -> repository=testpypi` | 3.10 / 3.11 / 3.12 / 3.13 / 3.14 | Linuxï¼ˆx86_64ã€aarch64ï¼‰ã€Windowsï¼ˆx86_64ï¼‰ã€macOSï¼ˆarm64ï¼‰ | wheel + sdist |
| PyPI | æŽ¨ `vX.Y.Z` æ ‡ç­¾æˆ–æ‰‹åŠ¨ `repository=pypi` | 3.10 / 3.11 / 3.12 / 3.13 / 3.14 | Linuxï¼ˆx86_64ã€aarch64ï¼‰ã€Windowsï¼ˆx86_64ï¼‰ã€macOSï¼ˆarm64ï¼‰ | wheel + sdist |

ä»Ž PyPI å®‰è£…ï¼š

```bash
pip install motorbridge
```

æºç å…œåº•å®‰è£…ï¼š

```bash
pip install --no-binary motorbridge motorbridge
```

### C) æŒ‰åˆ†å‘ç±»åž‹çœ‹åŠŸèƒ½è¾¹ç•Œ

| åˆ†å‘ç±»åž‹ | å…¸åž‹åœºæ™¯ | ä½ èƒ½åšä»€ä¹ˆ |
|---|---|---|
| ABI åŒ…ï¼ˆ`.deb/.tar.gz/.zip`ï¼‰ | C/C++ é›†æˆ | è°ƒç”¨ç¨³å®š C ABIã€ä½¿ç”¨ C++ RAII wrapperã€åµŒå…¥åŽŸç”Ÿæœºå™¨äººç³»ç»Ÿ |
| Python åŒ…ï¼ˆwheel/sdistï¼‰ | Python åº”ç”¨/å·¥å…· | ä½¿ç”¨ `Controller/Motor/Mode` API å’Œ `motorbridge-cli` |
| `motor_cli` äºŒè¿›åˆ¶åŒ… | è¿ç»´/å·¥åŽ‚/è”è°ƒ | ä¸ä¾èµ– Python ç›´æŽ¥åš CAN æ‰«æå’ŒæŽ§åˆ¶ |

### D) é¢å¤–è‡ªåŠ¨åŒ–åˆ†å‘æ¸ é“ï¼ˆå½“å‰å¯ç”¨ï¼‰

| æ¸ é“ | CI Workflow | è¾“å‡º |
|---|---|---|
| APT ä»“åº“ï¼ˆGitHub Pagesï¼‰ | `.github/workflows/apt-repo-publish.yml` | `https://<owner>.github.io/<repo>/apt` |

è¯´æ˜Žï¼š
- `.deb` å½“å‰é¢å‘ Linux x86_64ï¼›å…¶ä»– Linux ç›®æ ‡å»ºè®®ä½¿ç”¨ ABI `.tar.gz`ã€‚
- å½“å‰å‘å¸ƒçŸ©é˜µæ•…æ„ä¸äº§å‡º macOS x86_64 wheelã€‚
- è®¾å¤‡çŸ©é˜µå‚è€ƒï¼š`docs/zh/devices.md`ã€‚
- åˆ†å‘è‡ªåŠ¨åŒ–æ–‡æ¡£ï¼š`docs/zh/distribution_channels.md`ã€‚

