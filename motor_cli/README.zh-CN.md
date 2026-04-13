# motor_cliï¼ˆä¸­æ–‡ï¼‰

Rust `motor_cli` çš„å…¨å‚æ•°å®Œæ•´è¯´æ˜Žã€‚

- Crate: `motor_cli`
- æŽ¨èï¼ˆrelease åŽ‹ç¼©åŒ…ï¼‰ï¼š`./bin/motor_cli [å‚æ•°...]`
- å¯é€‰ï¼ˆæºç ç¼–è¯‘åŽï¼‰ï¼š`./target/release/motor_cli [å‚æ•°...]`

## ä¼˜å…ˆä½¿ç”¨ Release äºŒè¿›åˆ¶

å…ˆä»Ž GitHub Releases ä¸‹è½½å¹¶è§£åŽ‹å¯¹åº”åŒ…ï¼ˆä¾‹å¦‚ `motor-cli-vX.Y.Z-linux-x86_64.tar.gz`ï¼‰ï¼Œå†ç›´æŽ¥è¿è¡Œï¼š

```bash
./bin/motor_cli -h
./bin/motor_cli --vendor damiao --mode scan --start-id 1 --end-id 16
```

å¦‚æžœä½ å¸Œæœ›ç›´æŽ¥è¾“å…¥ `motor_cli` å‘½ä»¤ï¼š

```bash
export PATH="$(pwd)/bin:$PATH"
motor_cli -h
```

## Damiao æŒ‡ä»¤ä¸Žå¯„å­˜å™¨è¿›é˜¶æ–‡æ¡£

- ä¸­æ–‡è¯¦è¡¨ï¼ˆæŒ‡ä»¤/å¯„å­˜å™¨/è°ƒå‚ï¼‰: `DAMIAO_API.zh-CN.md`
- English version: `DAMIAO_API.md`

## RobStride æŒ‡ä»¤ä¸Žå‚æ•°è¿›é˜¶æ–‡æ¡£

- ä¸­æ–‡è¯¦è¡¨ï¼ˆå‚æ•°/èƒ½åŠ›è¾¹ç•Œï¼‰: `ROBSTRIDE_API.zh-CN.md`
- English version: `ROBSTRIDE_API.md`

## MyActuator æŒ‡ä»¤ä¸Žæ¨¡å¼è¿›é˜¶æ–‡æ¡£

- ä¸­æ–‡è¯¦è¡¨ï¼ˆå‘½ä»¤/æ¨¡å¼/å‚æ•°ï¼‰: `MYACTUATOR_API.zh-CN.md`
- English version: `MYACTUATOR_API.md`

## HighTorque è¡¥å……è¯´æ˜Ž

- åè®®æ·±åº¦åˆ†æžæ–‡æ¡£ï¼š`../docs/zh/hightorque_protocol_analysis.md`
- å½“å‰ `vendor=hightorque` ä¸º åŽŸç”Ÿ ht_can v1.5.5 çš„â€œç›´è¿ž CANâ€æ¨¡å¼ï¼Œä¸æ˜¯å®˜æ–¹çš„â€œä¸²å£->CANboardâ€ä¼ è¾“é“¾è·¯ã€‚

## CAN è°ƒè¯•å…¥å£

- Linux `slcan` + Windows `pcan` ä¸“ä¸šæŽ’éšœï¼š`../docs/zh/can_debugging.md`
- English guide: `../docs/en/can_debugging.md`

## ä¼ è¾“æ ‡è¯†

- `[STD-CAN]` => `--transport auto|socketcan`
- `[CAN-FD]` => `--transport socketcanfd`ï¼ˆä»… Linuxï¼›Hexfellow å¿…é¡»ä½¿ç”¨ï¼‰
- `[DM-SERIAL]` => `--transport dm-serial`ï¼ˆä»… Damiaoï¼‰

å½“å‰çŠ¶æ€ï¼š
- Hexfellowï¼š`socketcanfd` è·¯å¾„å·²å®žæµ‹ï¼Œç»Ÿä¸€ `mit` / `pos-vel` å¯ç”¨ã€‚
- HighTorqueï¼šæ ‡å‡† CAN ä¸‹ç»Ÿä¸€ `mit` / `vel` å·²å®žæµ‹å¯ç”¨ï¼ˆåè®®å±‚å¿½ç•¥ `kp/kd`ï¼‰ã€‚
- Damiaoï¼šç»Ÿä¸€ `mit` / `pos-vel` / `vel` / `force-pos` çš„åŸºçº¿å®žçŽ°ã€‚

## 1. å‚æ•°è§£æžè§„åˆ™

- ä»…è§£æž `--key value` å½¢å¼ã€‚
- å•ç‹¬å¼€å…³ï¼ˆå¦‚ `--help`ï¼‰ä¼šæŒ‰å€¼ `1` å¤„ç†ã€‚
- ID ç±»å‚æ•°æ”¯æŒåè¿›åˆ¶ï¼ˆå¦‚ `20`ï¼‰ä¸Žåå…­è¿›åˆ¶ï¼ˆå¦‚ `0x14`ï¼‰ã€‚
- æœªè¢«ä»£ç ä½¿ç”¨çš„å‚æ•°å³ä½¿ä¼ å…¥ï¼Œä¹Ÿä¸ä¼šç”Ÿæ•ˆã€‚

### 1.1 ç»Ÿä¸€è°ƒç”¨èŒƒå¼ï¼ˆCLI å¤§ä¸€ç»Ÿï¼‰

æ‰€æœ‰å“ç‰Œéƒ½éµå¾ªåŒä¸€ä¸ªè°ƒç”¨éª¨æž¶ï¼Œåªæ˜¯ `vendor/model/mode` ä¸Žé™„åŠ å‚æ•°ä¸åŒï¼š

```bash
motor_cli \
  --vendor <damiao|robstride|hightorque|myactuator|all> \
  --transport <auto|socketcan|socketcanfd|dm-serial> \
  --channel <can0|slcan0|can0@1000000...> \
  --model <model-name> \
  --motor-id <id> --feedback-id <id> \
  --mode <mode-name> \
  [æ¨¡å¼å‚æ•°...] \
  --loop <n> --dt-ms <ms>
```

è¯´æ˜Žï¼š
- `socketcanfd` ä¸º Hexfellow å¿…éœ€é“¾è·¯ï¼›Damiao å¯æŒ‰åž‹å·åš CAN-FD éªŒè¯ï¼›`dm-serial` ä»… Damiao å¯ç”¨ã€‚
- `vendor=all` å½“å‰ä»…ç”¨äºŽç»Ÿä¸€æ‰«æï¼ˆ`--mode scan`ï¼‰ã€‚

### 1.2 é€šç”¨å‚æ•°è¯­ä¹‰ï¼ˆå…ˆç†è§£è¿™äº›ï¼‰

| å‚æ•° | è¯­ä¹‰ |
|---|---|
| `--vendor` | é€‰æ‹©å“ç‰Œé©±åŠ¨å®žçŽ°ï¼ˆç»Ÿä¸€å…¥å£ä¸‹å‘åˆ°ä¸åŒ vendor backendï¼‰ |
| `--transport` | é€‰æ‹©ä¼ è¾“å±‚ï¼ˆæ ‡å‡† CAN æˆ– Damiao ä¸²å£æ¡¥ï¼‰ |
| `--channel` | CAN é€šé“åï¼ˆLinux ä¸ºç½‘å¡åï¼›Windows å¯å¸¦ `@bitrate`ï¼‰ |
| `--model` | åž‹å·åç§°ï¼Œç”¨äºŽè¯¥å“ç‰Œä¸‹çš„é™å€¼/èƒ½åŠ›è¾¹ç•Œä¸Žç¼–ç æ˜ å°„ |
| `--motor-id` | ç›®æ ‡ç”µæœº IDï¼ˆå‘é€å‘½ä»¤ç›®æ ‡ï¼‰ |
| `--feedback-id` | åé¦ˆå¸§ IDï¼ˆæŽ¥æ”¶çŠ¶æ€æ¥æºï¼‰ |
| `--mode` | æŽ§åˆ¶/æŸ¥è¯¢åŠ¨ä½œç±»åž‹ï¼ˆä¸åŒå“ç‰Œæ”¯æŒé›†åˆä¸åŒï¼‰ |
| `--loop` / `--dt-ms` | å¾ªçŽ¯å‘é€æ¬¡æ•° / å‘¨æœŸ |
| `--ensure-mode` | æŽ§åˆ¶å‰æ˜¯å¦è‡ªåŠ¨åˆ‡æŽ§åˆ¶æ¨¡å¼ï¼ˆDamiao ç­‰æ”¯æŒï¼‰ |

### 1.3 å„å“ç‰Œå‚æ•°å˜é‡æ€Žä¹ˆä¼ ï¼ˆç»Ÿä¸€è°ƒç”¨ä¸‹çš„å·®å¼‚ï¼‰

| å“ç‰Œ | `--model` ä¼ å…¥ | `--motor-id` / `--feedback-id` ä¼ å…¥ | å¸¸ç”¨ `--mode` |
|---|---|---|---|
| Damiao | å¿…ä¼ ä¸”å»ºè®®æŒ‰ç”µæœºçœŸå®žåž‹å·ï¼ˆæ··åž‹åœºæ™¯ä¸è¦å†™æ­»ä¸€ä¸ª modelï¼‰ | `motor-id` ä¸Ž `feedback-id` éƒ½éœ€è¦æŒ‰å®žé™…è®¾å¤‡ä¼ å…¥ | `scan`ã€`enable`ã€`disable`ã€`mit`ï¼ˆå½“å‰ä¸²å£æ¡¥å»ºè®®è¿™å››ä¸ªï¼‰ |
| RobStride | ä¼  `rs-00/01...` ç­‰ | `motor-id` å¿…ä¼ ï¼›`feedback-id` å¸¸ç”¨ `0xFD` | `ping`ã€`scan`ã€`mit`ã€`vel`ã€`read-param`ã€`write-param` |
| HighTorque | ä¼  `hightorque`ï¼ˆhintï¼‰ | æŒ‰è®¾å¤‡ ID ä¼ å…¥ | `read`ã€`mit`ã€`pos`ã€`vel`ã€`tqe`ã€`scan` ç­‰ |
| MyActuator | ä¼ è¿è¡Œæ—¶åž‹å·å­—ç¬¦ä¸²ï¼ˆé»˜è®¤ `X8`ï¼‰ | æ ‡å‡† 11-bit è§„åˆ™ï¼ˆå¸¸ç”¨ `0x140+id` / `0x240+id`ï¼‰ | `status`ã€`scan`ã€`current`ã€`vel`ã€`pos`ã€`enable/disable` |
| all | åˆ†å“ç‰Œ hintï¼ˆ`--damiao-model` ç­‰ï¼‰ | ä»…æ‰«æåœºæ™¯ä½¿ç”¨ | `scan` |

## 2. é¡¶å±‚é€šç”¨å‚æ•°ï¼ˆæ‰€æœ‰ vendorï¼‰

| å‚æ•° | ç±»åž‹ | é»˜è®¤å€¼ | è¯´æ˜Ž |
|---|---|---|---|
| `--help` | flag | å…³é—­ | è¾“å‡ºå¸®åŠ©å¹¶é€€å‡º |
| `--vendor` | string | `damiao` | `damiao` / `robstride` / `hightorque` / `myactuator` / `hexfellow` / `all` |
| `--transport` | string | `auto` | `auto` / `socketcan` / `socketcanfd` / `dm-serial`ï¼ˆ`socketcanfd` ä¸º Hexfellow å¿…éœ€ï¼›`dm-serial` ä»… Damiaoï¼‰ |
| `--channel` | string | `can0` | Linux：SocketCAN 网卡名（`can0`/`slcan0`）；Windows（PCAN 后端）：`can0`/`can1`，可加 `@bitrate`（如 `can0@1000000`）；macOS（PCBUSB 后端）：`can0`/`can1` |
| `--serial-port` | string | `/dev/ttyACM0` | `--transport dm-serial` æ—¶ä½¿ç”¨ |
| `--serial-baud` | u64 | `921600` | `--transport dm-serial` æ—¶ä½¿ç”¨ |
| `--model` | string | æŒ‰ vendor å†³å®š | Damiao é»˜è®¤ `4340`ï¼›RobStride é»˜è®¤ `rs-00`ï¼›HighTorque é»˜è®¤ `hightorque`ï¼›MyActuator é»˜è®¤ `X8` |
| `--motor-id` | u16(hex/dec) | `0x01` | ç”µæœº CAN ID |
| `--feedback-id` | u16(hex/dec) | æŒ‰ vendor å†³å®š | Damiao é»˜è®¤ `0x11`ï¼›RobStride é»˜è®¤ `0xFD`ï¼›HighTorque é»˜è®¤ `0x01`ï¼›MyActuator é»˜è®¤ `0x241`ï¼ˆmotor-id=1ï¼‰ |
| `--mode` | string | æŒ‰ vendor å†³å®š | Damiao é»˜è®¤ `mit`ï¼›RobStride é»˜è®¤ `ping`ï¼›HighTorque é»˜è®¤ `read`ï¼›MyActuator é»˜è®¤ `status`ï¼›`all` é»˜è®¤ `scan` |
| `--loop` | u64 | `1` | æŽ§åˆ¶å¾ªçŽ¯æ¬¡æ•° |
| `--dt-ms` | u64 | `20` | å¾ªçŽ¯é—´éš”æ¯«ç§’ |
| `--ensure-mode` | `0/1` | `1` | æŽ§åˆ¶å‰è‡ªåŠ¨åˆ‡æ¨¡å¼ |

### 2.1 é€šé“é€ŸæŸ¥ï¼ˆ`--channel`ï¼‰

- Linux SocketCANï¼š
  - ç›´æŽ¥ä½¿ç”¨ç½‘å¡åï¼š`can0`ã€`can1`ã€`slcan0`ã€‚
  - æ³¢ç‰¹çŽ‡åœ¨ç½‘å¡åˆå§‹åŒ–é˜¶æ®µè®¾ç½®ï¼ˆ`ip link` / `slcand`ï¼‰ï¼Œä¸è¦å†™åˆ° `--channel`ã€‚
  - `can0@1000000` åœ¨ Linux SocketCAN ä¸‹æ— æ•ˆã€‚
- Windows PCANï¼š
  - `can0` æ˜ å°„ `PCAN_USBBUS1`ï¼Œ`can1` æ˜ å°„ `PCAN_USBBUS2`ã€‚
  - æ”¯æŒå¯é€‰æ³¢ç‰¹çŽ‡åŽç¼€ï¼š`can0@1000000`ã€‚
- macOS PCBUSB（PCAN 后端）：
  - `can0` 映射 `PCAN_USBBUS1`，`can1` 映射 `PCAN_USBBUS2`。
  - 需先安装 `libPCBUSB.dylib`（见仓库根目录 `README.zh-CN.md` 的 macOS 章节）。


### 2.2 Damiao ä¸²å£æ¡¥é€ŸæŸ¥ï¼ˆ`--transport dm-serial`ï¼‰

- è¯¥é“¾è·¯ä¸ºé€‚é…å™¨ç§æœ‰è·¯å¾„ï¼Œé¢å‘ Damiao ç”µæœºã€‚
- å¸¸ç”¨å‚æ•°ï¼š`--transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600`ã€‚
- `dm-serial` æ¨¡å¼ä¸‹ï¼Œä¼ è¾“å±‚åˆ›å»ºä¼šå¿½ç•¥ `--channel`ã€‚
- `dm-serial` ä»…æ”¹å˜â€œä¼ è¾“å±‚â€ï¼ˆèµ°ä¸²å£æ¡¥ï¼‰ï¼ŒDamiao çš„ä¸šåŠ¡å‚æ•°ä¸Žæ¨¡å¼æŽ¥å£ä¿æŒä¸€è‡´ï¼ˆ`--mode`ã€`--motor-id`ã€`--feedback-id`ã€`--verify-model`ã€`--ensure-mode` ç­‰ï¼‰ã€‚

### 2.3 Damiao ç‹¬ç«‹ CAN-FD é“¾è·¯é€ŸæŸ¥ï¼ˆ`--transport socketcanfd`ï¼‰

- è¯¥é“¾è·¯ä¸º Linux ä¸“ç”¨ï¼Œå¹¶ä¸Žç»å…¸ `socketcan` é“¾è·¯å¹¶å­˜ã€‚
- å¸¸ç”¨å‚æ•°ï¼š`--transport socketcanfd --channel can0`ã€‚
- ä½¿ç”¨å‰å…ˆç¡®ä¿ç½‘å£å¤„äºŽ FD æ¨¡å¼ï¼ˆ`scripts/canfd_restart.sh can0`ï¼‰ã€‚
- å½“å‰çŠ¶æ€ï¼šé“¾è·¯å·²æŽ¥å…¥ï¼Œå°šæœªæ ‡æ³¨â€œå·²å®Œæˆ CAN-FD ç”µæœºéªŒè¯â€çš„åž‹å·åˆ—è¡¨ã€‚

## 3. vendor=`damiao`

### 3.1 æ”¯æŒæ¨¡å¼

- `scan`
- `enable`
- `disable`
- `mit`
- `pos-vel`
- `vel`
- `force-pos`

### 3.2 Damiao ä¸“ç”¨å‚æ•°

| å‚æ•° | ç±»åž‹ | é»˜è®¤å€¼ | ä½œç”¨èŒƒå›´ | è¯´æ˜Ž |
|---|---|---|---|---|
| `--verify-model` | `0/1` | `1` | éž scan | æ ¡éªŒ PMAX/VMAX/TMAX ä¸Ž `--model` ä¸€è‡´ |
| `--verify-timeout-ms` | u64 | `500` | éž scan | åž‹å·æ¡æ‰‹è¯»å–è¶…æ—¶ |
| `--verify-tol` | f32 | `0.2` | éž scan | é™å€¼åŒ¹é…å®¹å·® |
| `--start-id` | u16 | `1` | scan | æ‰«æèµ·å§‹ IDï¼ˆ1..255ï¼‰ |
| `--end-id` | u16 | `255` | scan | æ‰«æç»“æŸ IDï¼ˆ1..255ï¼‰ |
| `--set-motor-id` | u16 å¯é€‰ | æ—  | æ”¹ ID æµç¨‹ | å†™ ESC_IDï¼ˆRID 8ï¼‰ |
| `--set-feedback-id` | u16 å¯é€‰ | æ—  | æ”¹ ID æµç¨‹ | å†™ MST_IDï¼ˆRID 7ï¼‰ |
| `--store` | `0/1` | `1` | æ”¹ ID æµç¨‹ | æ˜¯å¦ä¿å­˜å‚æ•° |
| `--verify-id` | `0/1` | `1` | æ”¹ ID æµç¨‹ | æ˜¯å¦å›žè¯» RID7/RID8 æ ¡éªŒ |

### 3.3 å„æ¨¡å¼æŽ§åˆ¶å‚æ•°

| æ¨¡å¼ | å‚æ•° | é»˜è®¤å€¼ |
|---|---|---|
| `mit` | `--pos --vel --kp --kd --tau` | `0 0 2 1 0` |
| `pos-vel` | `--pos --vlim` | `0 1.0` |
| `vel` | `--vel` | `0` |
| `force-pos` | `--pos --vlim --ratio` | `0 1.0 0.1` |
| `enable` / `disable` | æ— é¢å¤–å‚æ•° | n/a |

### 3.4 æ‰«æè¡Œä¸ºç»†èŠ‚

- æ‰«æé€»è¾‘æœ¬è´¨ä¸Šæ˜¯â€œåž‹å·æ— å…³â€çš„ï¼šå†…éƒ¨ä¼šéåŽ†å†…ç½® model-hint åˆ—è¡¨ã€‚
- æ¯ä¸ªå€™é€‰ ID ä¼šå°è¯•å¤šä¸ª feedback-hintï¼šæŽ¨æ–­å€¼ï¼ˆ`id+0x10`ï¼‰ã€ç”¨æˆ·ç»™å®š `--feedback-id`ã€`0x11`ã€`0x17`ã€‚
- ä¼˜å…ˆç”¨å¯„å­˜å™¨ï¼ˆRID 21/22/23ï¼‰æ£€æµ‹ï¼Œå¤±è´¥å†èµ°åé¦ˆå›žé€€æ£€æµ‹ã€‚

### 3.5 Damiao ç¤ºä¾‹

```bash
# æ‰«æ 1..16
motor_cli \
  --vendor damiao --channel can0 --mode scan --start-id 1 --end-id 16
# [STD-CAN]

# MIT æŽ§åˆ¶
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --pos 1.57 --vel 2.0 --kp 35 --kd 1.2 --tau 0.3 --loop 120 --dt-ms 20
# [STD-CAN]

# é€šè¿‡ Damiao ä¸²å£æ¡¥æ‰§è¡Œ MIT
motor_cli \
  --vendor damiao --transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600 \
  --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --verify-model 0 --ensure-mode 0 \
  --pos 1.0 --vel 0 --kp 2 --kd 1 --tau 0 --loop 80 --dt-ms 20
# [DM-SERIAL]

# ä½ç½®é€Ÿåº¦æŽ§åˆ¶
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x04 --feedback-id 0x14 \
  --mode pos-vel --pos 3.14 --vlim 4.0 --loop 120 --dt-ms 20
# [STD-CAN]

# æ”¹ ID + ä¿å­˜ + æ ¡éªŒ
motor_cli \
  --vendor damiao --channel can0 --model 4310 --motor-id 0x01 --feedback-id 0x11 \
  --set-motor-id 0x04 --set-feedback-id 0x14 --store 1 --verify-id 1
```

### 3.6 Damiao ä¸²å£æ¡¥å®Œæ•´æŽ¥å£ä¸Žç”¨æ³•ï¼ˆ`--transport dm-serial`ï¼‰

å…ˆå®šä¹‰å…¬å…±å‰ç¼€ï¼ˆå»ºè®®ï¼‰ï¼š

```bash
DM_SERIAL="--vendor damiao --transport dm-serial --serial-port /dev/ttyACM1 --serial-baud 921600 --model 4310"
```

#### 3.6.1 ä¸²å£æ¡¥ä¸‹å¿…ç”¨/å¸¸ç”¨å‚æ•°

| å‚æ•° | æ˜¯å¦å»ºè®®æ˜¾å¼ä¼ å…¥ | è¯´æ˜Ž |
|---|---|---|
| `--transport dm-serial` | å¿…é¡» | åˆ‡åˆ° Damiao ä¸²å£æ¡¥é“¾è·¯ |
| `--serial-port` | å¿…é¡» | ä¸²å£è®¾å¤‡ï¼Œå¦‚ `/dev/ttyACM1` |
| `--serial-baud` | å¿…é¡» | ä¸²å£æ³¢ç‰¹çŽ‡ï¼Œå¸¸ç”¨ `921600` |
| `--channel` | å¯çœç•¥ | è¯¥æ¨¡å¼ä¸‹ä¼šè¢«å¿½ç•¥ |
| `--motor-id` / `--feedback-id` | æŽ§åˆ¶æ—¶å¿…é¡» | ä¸Žæ‰«æå‘½ä¸­ç»“æžœä¸€è‡´ |
| `--verify-model` | å»ºè®®æŒ‰çŽ°åœºå¼€å…³ | è‹¥æ¡æ‰‹é“¾è·¯ä¸ç¨³å®šå¯å…ˆè®¾ `0` åšè”é€šéªŒè¯ |
| `--ensure-mode` | å»ºè®®æŒ‰çŽ°åœºå¼€å…³ | è‹¥ç”µæœºæ¨¡å¼åˆ‡æ¢æµç¨‹ä¸ç¨³å®šå¯å…ˆè®¾ `0` |

> å½“å‰ä¸²å£æ¡¥åœºæ™¯å¯¹å¤–æŽ¨èä»…ä½¿ç”¨ï¼š`scan` / `enable` / `disable` / `mit`ã€‚

#### 3.6.2 ä¸²å£æ¡¥ä¸‹å¸¸ç”¨å››æ¨¡å¼å‘½ä»¤æ¨¡æ¿

```bash
# 1) æ‰«æ
motor_cli $DM_SERIAL --mode scan --start-id 1 --end-id 16

# 2) ä½¿èƒ½
motor_cli $DM_SERIAL --motor-id 0x04 --feedback-id 0x14 --mode enable --verify-model 0 --loop 1

# 3) å¤±èƒ½
motor_cli $DM_SERIAL --motor-id 0x04 --feedback-id 0x14 --mode disable --verify-model 0 --loop 1

# 4) MIT
motor_cli $DM_SERIAL --motor-id 0x04 --feedback-id 0x14 \
  --mode mit --verify-model 0 --ensure-mode 0 \
  --pos 0.5 --vel 0 --kp 2 --kd 1 --tau 0 --loop 80 --dt-ms 20
```

#### 3.6.3 æŽ¨èæµ‹è¯•é¡ºåº

1. `scan` å…ˆç¡®è®¤åœ¨çº¿ IDã€‚
2. `enable --loop 1` åšæœ€å°åŠ¨ä½œéªŒè¯ã€‚
3. `mit` å°æ­¥å‚æ•°ï¼ˆå° `pos`ã€ä¸­ä½Ž `kp/kd`ï¼‰éªŒè¯æŽ§åˆ¶é—­çŽ¯ã€‚
4. æœ€åŽå†ä¸Šä¸šåŠ¡å‚æ•°ä¸Žè¿žç»­å¾ªçŽ¯ã€‚

## 4. vendor=`robstride`

### 4.1 æ”¯æŒæ¨¡å¼

- `ping`
- `scan`
- `enable`
- `disable`
- `mit`
- `pos-vel`
- `vel`
- `read-param`
- `write-param`

### 4.2 RobStride ä¸“ç”¨å‚æ•°

| å‚æ•° | ç±»åž‹ | é»˜è®¤å€¼ | ä½œç”¨èŒƒå›´ | è¯´æ˜Ž |
|---|---|---|---|---|
| `--start-id` | u16 | `1` | scan | æ‰«æèµ·å§‹ IDï¼ˆ1..255ï¼‰ |
| `--end-id` | u16 | `255` | scan | æ‰«æç»“æŸ IDï¼ˆ1..255ï¼‰ |
| `--manual-vel` | f32 | `0.2` | scan å›žé€€ | ç›²æŽ¢é€Ÿåº¦ |
| `--manual-ms` | u64 | `200` | scan å›žé€€ | æ¯ä¸ª ID è„‰å†²æ—¶é•¿ |
| `--manual-gap-ms` | u64 | `200` | scan å›žé€€ | ID é—´éš” |
| `--set-motor-id` | u16 å¯é€‰ | æ—  | æ”¹ ID æµç¨‹ | è®¾ç½®æ–°è®¾å¤‡ ID |
| `--store` | `0/1` | `1` | æ”¹ ID æµç¨‹ | ä¿å­˜å‚æ•° |
| `--param-id` | u16 | å‚æ•°æ¨¡å¼å¿…å¡« | è¯»å†™å‚æ•° | å‚æ•° ID |
| `--param-value` | ç±»åž‹åŒ–å€¼ | å†™å‚æ•°å¿…å¡« | write-param | æŒ‰å‚æ•°å…ƒæ•°æ®è§£æž |

### 4.3 å„æ¨¡å¼æŽ§åˆ¶å‚æ•°

| æ¨¡å¼ | å‚æ•° | é»˜è®¤å€¼ |
|---|---|---|
| `mit` | `--pos --vel --kp --kd --tau` | `0 0 8 0.2 0` |
| `pos-vel` | `--pos --vlim [--kp]` | `0 1.0 [æ— ]` |
| `vel` | `--vel` | `0` |
| `enable` / `disable` | æ— é¢å¤–å‚æ•° | n/a |

è¯´æ˜Žï¼š

- RobStride ç»Ÿä¸€é«˜å±‚å½“å‰æ”¯æŒ `MIT` / `POS_VEL` / `VEL`ã€‚
- `TORQUE/CURRENT` ç›®å‰ä»æ˜¯å‚æ•°çº§èƒ½åŠ›ï¼ˆé€šè¿‡ `write-param` å†™ `iq_ref` ä¸Žé™å¹…å‚æ•°ï¼‰ï¼Œå°šæœªå¼€æ”¾ç»Ÿä¸€æ¨¡å¼ã€‚
- RobStride 的 `mit` 五个参数都生效：`--pos`、`--vel`、`--kp`、`--kd`、`--tau`。
- RobStride 的 `mit` 单位：`pos(rad)`、`vel(rad/s)`、`tau(Nm)`，`kp/kd` 为 MIT 闭环增益。
- RobStride 的 `pos-vel` 仅消费 `--pos`、`--vlim`、可选 `--kp`/`--loc-kp`。
- RobStride 的 `pos-vel` 会忽略 `--vel`、`--kd`、`--tau`（CLI 在传入时会打印 warning）。

### 4.4 æ‰«æè¡Œä¸ºç»†èŠ‚

- ç¬¬ä¸€é˜¶æ®µï¼šæ¯ä¸ª ID åš `ping` + å‚æ•°æŸ¥è¯¢æŽ¢æµ‹ã€‚
- å…¨èŒƒå›´æ— å‘½ä¸­æ—¶ï¼šè¿›å…¥ç›²æŽ¢é€Ÿåº¦è„‰å†²æ¨¡å¼ï¼ˆäººå·¥è§‚å¯Ÿæ˜¯å¦è½¬åŠ¨ï¼‰ã€‚
- å›žé€€é˜¶æ®µè‹¥æœ‰çŠ¶æ€åé¦ˆï¼Œä¹Ÿä¼šè®¡å…¥å‘½ä¸­ã€‚

### 4.5 RobStride ç¤ºä¾‹

```bash
# ping
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFD --mode ping

# æ‰«æ
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 255

# MIT æŽ§åˆ¶
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFD \
  --mode mit --pos 3.14 --vel 0 --kp 0.5 --kd 0.2 --tau 0 --loop 120 --dt-ms 20

# POS_VELï¼ˆæ˜ å°„åˆ°åŽŸç”Ÿ Positionï¼‰
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFD \
  --mode pos-vel --pos 1.0 --vlim 1.5 --loop 1 --dt-ms 20

# é€Ÿåº¦æ¨¡å¼
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFD \
  --mode vel --vel 2.0 --loop 100 --dt-ms 20

# è¯»å‚æ•°
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFD \
  --mode read-param --param-id 0x7005

# å†™å‚æ•°
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 20 --feedback-id 0xFD \
  --mode write-param --param-id 0x7005 --param-value 2

# æ”¹ IDï¼ˆæ—§ 1 -> æ–° 11ï¼‰å¹¶å­˜å‚
motor_cli \
  --vendor robstride --channel can0 --model rs-00 --motor-id 1 --feedback-id 0xFD \
  --set-motor-id 11 --store 1

# è®¾é›¶ï¼ˆå®žéªŒæ—¶åºï¼‰
motor_cli \
  --vendor robstride --channel can0 --model rs-00 --motor-id 11 --feedback-id 0xFD \
  --mode zero --zero-exp 1 --store 1
```

## 5. vendor=`all`

`vendor=all` å½“å‰ä»…æ”¯æŒ `--mode scan`ã€‚

### 5.1 all-scan é¢å¤–å‚æ•°

| å‚æ•° | é»˜è®¤å€¼ | è¯´æ˜Ž |
|---|---|---|
| `--damiao-model` | `4340P` | ä¼ ç»™ Damiao æ‰«ææµç¨‹çš„ model hint |
| `--robstride-model` | `rs-00` | ä¼ ç»™ RobStride æ‰«ææµç¨‹çš„ model hint |
| `--hightorque-model` | `hightorque` | ä¼ ç»™ HighTorque æ‰«ææµç¨‹çš„ model hint |
| `--myactuator-model` | `X8` | ä¼ ç»™ MyActuator æ‰«ææµç¨‹çš„ model hint |
| `--start-id` | `1` | åŒæ—¶ä¼ ç»™å„æ‰«ææµç¨‹ |
| `--end-id` | `255` | ä¼ ç»™ Damiao/RobStrideï¼›MyActuator ä¼šè‡ªåŠ¨æˆªæ–­åˆ° `32` |

### 5.2 ç¤ºä¾‹

```bash
motor_cli \
  --vendor all --channel can0 --mode scan --start-id 1 --end-id 255
```

## 5.3 vendor=`hightorque`ï¼ˆåŽŸç”Ÿ `ht_can` v1.5.5ï¼‰

- å½“å‰å®žçŽ°èµ° HighTorque åŽŸç”Ÿ `ht_can` v1.5.5 ç›´è¿ž CAN åè®®è·¯å¾„ã€‚
- ç”¨äºŽ SocketCANï¼ˆ`can0` ç­‰ï¼‰ç›´è¿žç”µæœºåœºæ™¯ã€‚
- HighTorque å®˜æ–¹ Panthera SDK çš„â€œUSB ä¸²å£ -> CANboard -> ç”µæœºâ€é“¾è·¯ä¸Žå½“å‰ CLI ç›´è¿ž CAN è·¯å¾„ç›¸äº’ç‹¬ç«‹ã€‚
- æ”¯æŒæ¨¡å¼ï¼š`scan | read | ping | mit | pos | vel | tqe | pos-vel-tqe | volt | cur | stop | brake | rezero | conf-write | timed-read`ã€‚
- ç»Ÿä¸€å•ä½æŽ¥å£ï¼š
  - `--pos` ä¸º `rad`
  - `--vel` ä¸º `rad/s`
  - `--tau` ä¸º `Nm`
  - `--kp`ã€`--kd` ä¸ºç»Ÿä¸€ MIT å‚æ•°ç­¾åä¿ç•™ï¼Œ`ht_can` åè®®æœ¬èº«ä¸ä½¿ç”¨ã€‚
  - åŽŸå§‹è°ƒè¯•å‚æ•°ï¼š`--raw-pos`ã€`--raw-vel`ã€`--raw-tqe`ã€‚

## 6. vendor=`myactuator`

### 6.1 æ”¯æŒæ¨¡å¼

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

### 6.2 MyActuator ä¸“ç”¨å‚æ•°

| å‚æ•° | ç±»åž‹ | é»˜è®¤å€¼ | ä½œç”¨èŒƒå›´ | è¯´æ˜Ž |
|---|---|---|---|---|
| `--start-id` | u16 | `1` | scan | æ‰«æèµ·å§‹ IDï¼ˆ1..32ï¼‰ |
| `--end-id` | u16 | `32` | scan | æ‰«æç»“æŸ IDï¼ˆ1..32ï¼Œä¼ å…¥å¤§äºŽ 32 ä¼šè‡ªåŠ¨æˆªæ–­ï¼‰ |
| `--current` | f32 | `0.0` | current | ç”µæµç›®æ ‡å€¼ï¼ˆAï¼‰ |
| `--vel` | f32 | `0.0` | vel | é€Ÿåº¦ç›®æ ‡å€¼ï¼ˆrad/sï¼Œå†…éƒ¨è½¬æ¢ä¸º deg/sï¼‰ |
| `--pos` | f32 | `0.0` | pos | ç»å¯¹ä½ç½®ç›®æ ‡å€¼ï¼ˆradï¼Œå†…éƒ¨è½¬æ¢ä¸º degï¼‰ |
| `--max-speed` | f32 | `8.726646` | pos | ä½ç½®æ¨¡å¼æœ€å¤§é€Ÿåº¦ï¼ˆrad/sï¼Œå†…éƒ¨è½¬æ¢ï¼‰ |

çŠ¶æ€è¾“å‡ºè¯´æ˜Žï¼š

- `angle` æ¥è‡ª `0x9C` çŠ¶æ€2è¿‘åœˆè§’ã€‚
- `mt_angle` æ¥è‡ª `0x92` å¤šåœˆè§’ï¼Œç»å¯¹ä½ç½®åˆ¤å®šåº”ä¼˜å…ˆçœ‹å®ƒã€‚

### 6.3 MyActuator ç¤ºä¾‹

```bash
# æ‰«æ 1..32
motor_cli \
  --vendor myactuator --channel can0 --mode scan --start-id 1 --end-id 32

# è¿žç»­çŠ¶æ€è¯»å–
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode status --loop 40 --dt-ms 50

# é€Ÿåº¦æ¨¡å¼
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode vel --vel 0.5236 --loop 100 --dt-ms 20

# ä½ç½®æ¨¡å¼
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode pos --pos 3.1416 --max-speed 5.236 --loop 1

# å°†å½“å‰ä½ç½®è®¾ä¸ºé›¶ç‚¹ï¼ˆæŒä¹…ç”Ÿæ•ˆéœ€æ–­ç”µé‡å¯ï¼‰
motor_cli \
  --vendor myactuator --channel can0 --model X8 --motor-id 1 --feedback-id 0x241 \
  --mode set-zero --loop 1
```

## 7. vendor=`hexfellow`

é“¾è·¯é™åˆ¶ï¼š
- Hexfellow åœ¨æœ¬ä»“åº“æŒ‰â€œä»… CAN-FDâ€æŽ¥å…¥ï¼ˆ`--transport socketcanfd`ï¼‰ã€‚
- å½“å‰æ”¯æŒèŒƒå›´ï¼š`scan / status / pos-vel / mit / enable / disable`ã€‚
- å½“å‰çŠ¶æ€ï¼šé“¾è·¯å·²æŽ¥å…¥ï¼Œç”µæœºéªŒè¯çŸ©é˜µå¾…è¡¥ã€‚

### 7.1 Hexfellow ç¤ºä¾‹

```bash
# æ‰«æ ID
motor_cli \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --mode scan --start-id 1 --end-id 32

# çŠ¶æ€æŸ¥è¯¢
motor_cli \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --model hexfellow --motor-id 1 --feedback-id 0 \
  --mode status

# ä½ç½®é€Ÿåº¦ï¼ˆpos å•ä½ radï¼Œvlim å•ä½ rad/sï¼‰
motor_cli \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --model hexfellow --motor-id 1 --feedback-id 0 \
  --mode pos-vel --pos 3.1415926 --vlim 2.0

# MITï¼ˆpos/vel å•ä½ rad/rad/sï¼‰
motor_cli \
  --vendor hexfellow --transport socketcanfd --channel can0 \
  --model hexfellow --motor-id 1 --feedback-id 0 \
  --mode mit --pos 0.0 --vel 0.0 --kp 1000 --kd 100 --tau 0
```

## 8. å®žç”¨å»ºè®®

- Damiao æ”¹ ID å»ºè®®å§‹ç»ˆä½¿ç”¨ `--store 1 --verify-id 1`ã€‚
- è‹¥æ‰«æå¶å‘æ¼æ£€ï¼Œé‡å¯ CAN åŽé‡è¯•ã€‚
- RobStride æ²¡æœ‰ CLI çš„ `send_pos_vel` æ¨¡å¼ï¼Œè¯·ç”¨ `mit` æˆ– `vel`ã€‚

## 已验证能力矩阵（Damiao + RobStride，2026-04）

| 能力 | Damiao | RobStride |
|---|---|---|
| Scan | 支持 | 支持 |
| Ping/在线探测 | 支持（scan/寄存器路径） | 支持（`ping`） |
| Enable/Disable | 支持 | 支持 |
| MIT (`pos/vel/kp/kd/tau`) | 支持 | 支持 |
| POS_VEL 统一模式 | 支持 | 支持（映射到原生 Position） |
| VEL 统一模式 | 支持 | 支持 |
| 参数读写 | 支持 | 支持 |
| 置零 | 支持（建议先 disable） | 支持（实验序列；ACK 可能偶发超时） |
| 改电机 ID | 支持（`--set-motor-id`） | 支持（`--set-motor-id`） |
| 改反馈 ID | 支持（`--set-feedback-id`） | 不支持（RobStride 通过 `--feedback-id` 设定 host 路径） |

说明：
- RobStride 默认 `--feedback-id` 为 `0xFD`，内部会回退探测 `0xFF/0xFE`。
- RobStride 的 `pos-vel` 下 `--vel/--kd/--tau` 为无效参数，仅告警不报错。
- MyActuator è‹¥ `0x9A` è¿”å›žé”™è¯¯ç  `0x0004`ï¼ˆæ¬ åŽ‹ï¼‰ï¼Œç”µæœºä¼šåœ¨çº¿ä½†ä¸è½¬ï¼Œéœ€è¦å…ˆæ¢å¤ä¾›ç”µç”µåŽ‹ã€‚






