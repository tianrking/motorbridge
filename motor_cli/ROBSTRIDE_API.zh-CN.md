# RobStride API ä¸Žå‚æ•°å‚è€ƒï¼ˆå®Œæ•´ç‰ˆï¼‰

<!-- channel-compat-note -->
## é€šé“å…¼å®¹è¯´æ˜Žï¼ˆPCAN + slcan + Damiao ä¸²å£æ¡¥ï¼‰

- Linux SocketCAN ç›´æŽ¥ä½¿ç”¨ç½‘å¡åï¼š`can0`ã€`can1`ã€`slcan0`ã€‚
- ä¸²å£ç±» USB-CAN éœ€å…ˆåˆ›å»ºå¹¶æ‹‰èµ· `slcan0`ï¼š`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`ã€‚
- ä»… Damiao å¯é€‰ä¸²å£æ¡¥é“¾è·¯ï¼š`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`ã€‚
- Linux SocketCAN ä¸‹ `--channel` ä¸è¦å¸¦ `@bitrate`ï¼ˆä¾‹å¦‚ `can0@1000000` æ— æ•ˆï¼‰ã€‚
- Windowsï¼ˆPCAN åŽç«¯ï¼‰ä¸­ï¼Œ`can0/can1` æ˜ å°„ `PCAN_USBBUS1/2`ï¼Œå¯é€‰ `@bitrate` åŽç¼€ã€‚


æœ¬é¡µæ˜¯ `motorbridge` å½“å‰ RobStride æŽ§åˆ¶ã€å‚æ•°è¯»å†™ã€ä»¥åŠèƒ½åŠ›è¾¹ç•Œçš„å®Œæ•´å®žç”¨æ–‡æ¡£ã€‚

> English version: [ROBSTRIDE_API.md](ROBSTRIDE_API.md)

## 1ï¼‰é€šç”¨è®¾å¤‡å‚æ•°

| å‚æ•° | å«ä¹‰ | å¸¸ç”¨å€¼ |
|---|---|---|
| `channel` | CAN æŽ¥å£å | `can0` |
| `model` | RobStride åž‹å·å­—ç¬¦ä¸² | `rs-00`ã€`rs-06` |
| `motor-id` | è®¾å¤‡ ID | å¦‚ `127` |
| `feedback-id` | å‘½ä»¤å¸§é‡Œçš„ä¸»æœº/åé¦ˆ ID | å¸¸ç”¨ `0xFF` |
| `loop` | å‘¨æœŸæŽ§åˆ¶å‘é€æ¬¡æ•° | `20`~`100` |
| `dt-ms` | å‘¨æœŸå‘é€é—´éš” | `20`~`50` |

## 2ï¼‰`motor_cli` çš„ RobStride æ¨¡å¼

å½“å‰æ”¯æŒï¼š

- `ping`
- `scan`
- `enable`
- `disable`
- `mit`
- `pos-vel`
- `vel`
- `read-param`
- `write-param`

å¤§ä¸€ç»Ÿâ€œå››åè®®â€æ˜ å°„çŠ¶æ€ï¼š

| å¤§ä¸€ç»Ÿèƒ½åŠ› | RobStride çŠ¶æ€ | è¯´æ˜Ž |
|---|---|---|
| `MIT` | å·²æ”¯æŒ | åŽŸç”Ÿ operation-control å¸§ |
| `POS_VEL` | å·²æ”¯æŒ | æ˜ å°„åˆ° `run_mode=1` + `0x7017/0x7016` |
| `VEL` | å·²æ”¯æŒ | æ˜ å°„åˆ° `run_mode=2` + `0x700A` |
| `TORQUE/CURRENT` | ä»…å‚æ•°çº§ | å°šæ— ç»Ÿä¸€é«˜å±‚æ¨¡å¼ï¼›é€šè¿‡ `write-param` å†™ `iq_ref`/é™å¹…å‚æ•° |

### 2.1 Ping

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode ping
```

### 2.2 MIT

MIT 映射说明（统一接口 -> RobStride 原生）：

- 有效参数：`--pos`、`--vel`、`--kp`、`--kd`、`--tau`（五个都生效）。
- 单位约定：
  - `--pos`：`rad`
  - `--vel`：`rad/s`
  - `--tau`：`Nm`
  - `--kp`、`--kd`：MIT 闭环增益

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode mit --pos 0 --vel 0 --kp 0.5 --kd 0.2 --tau 0 --loop 40 --dt-ms 50
```

### 2.3 é€Ÿåº¦æ¨¡å¼

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode vel --vel 0.3 --loop 40 --dt-ms 50
```

### 2.4 ä½ç½®æ¨¡å¼ï¼ˆç»Ÿä¸€ `pos-vel` æ˜ å°„ï¼‰

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode pos-vel --pos 1.0 --vlim 1.5 --loop 1 --dt-ms 20
```

è¯´æ˜Žï¼š

- ç»Ÿä¸€ `pos-vel` å·²æ˜ å°„ä¸º RobStride åŽŸç”Ÿ Position é“¾è·¯ï¼š
  - `run_mode=1`ï¼ˆPositionï¼‰
  - å†™ `0x7017`ï¼ˆ`limit_spd`ï¼‰ä¸º `--vlim`
  - å¯é€‰å†™ `0x701E`ï¼ˆ`loc_kp`ï¼‰æ¥è‡ª `--loc-kp` æˆ– `--kp`
  - å†™ `0x7016`ï¼ˆ`loc_ref`ï¼‰ä¸º `--pos`
- `--vel`ã€`--kd`ã€`--tau` ä¸å±žäºŽåŽŸç”Ÿ Position æ¨¡å¼ï¼Œåœ¨ `--mode pos-vel` ä¸‹ä¼šè¢«å¿½ç•¥ã€‚

### 2.5 ä¸¤ç§ä½¿ç”¨æ–¹å¼ï¼ˆç»Ÿä¸€å°è£… / åŽŸç”Ÿå‚æ•°ï¼‰

- ç»Ÿä¸€å°è£…æ–¹å¼ï¼ˆæŽ¨èä¸Šå±‚ä¸šåŠ¡ä½¿ç”¨ï¼‰ï¼š
  - `--mode mit`
  - `--mode pos-vel`ï¼ˆå·²æ˜ å°„åˆ°åŽŸç”Ÿ Positionï¼‰
  - `--mode vel`
- åŽŸç”Ÿæ–¹å¼ï¼ˆè°ƒè¯•/åè®®çº§éªŒè¯ï¼‰ï¼š
  - `--mode read-param --param-id ...`
  - `--mode write-param --param-id ... --param-value ...`
  - å…¸åž‹é“¾è·¯ï¼šå…ˆå†™ `run_mode(0x7005)`ï¼Œå†å†™å¯¹åº”ç›®æ ‡å‚æ•°ï¼ˆå¦‚ `loc_ref/spd_ref`ï¼‰

## 3ï¼‰æ‰«æä¸Žæ”¹ ID

### 3.1 æ‰«æ

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --mode scan --start-id 1 --end-id 255
```

è¯´æ˜Žï¼š

- ç¬¬ä¸€é˜¶æ®µï¼š`ping + å‚æ•°æŸ¥è¯¢æŽ¢æµ‹`ã€‚
- è‹¥å…¨èŒƒå›´æ—  ping å‘½ä¸­ï¼šè‡ªåŠ¨å›žé€€åˆ°ç›²æŽ¢è„‰å†²ï¼ˆè§‚å¯Ÿç”µæœºæ˜¯å¦è½¬åŠ¨ï¼‰ã€‚
  - `--manual-vel`ï¼ˆé»˜è®¤ `0.2`ï¼‰
  - `--manual-ms`ï¼ˆé»˜è®¤ `200`ï¼‰
  - `--manual-gap-ms`ï¼ˆé»˜è®¤ `200`ï¼‰

### 3.2 æ”¹è®¾å¤‡ ID

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 \
  --motor-id 127 --feedback-id 0xFD --set-motor-id 126 --store 1
```

与上位机抓包对齐的报文说明：

- 改 ID 使用 `comm_type=7`。
- 该路径下扩展 ID 组成为：
  - `0x07 [new_id] [host_id] [old_id]`
  - 例如（`old_id=1`、`new_id=11`、`host_id=0xFD`）：`0x070BFD01`
- 数据区优先使用最近一次 `ping` 的 UUID token（若拿不到 token 则回退为全 0）。

## 4ï¼‰å¸¸ç”¨å‚æ•° ID

| Param ID | åç§° | ç±»åž‹ | å«ä¹‰ |
|---|---|---|---|
| `0x7005` | `run_mode` | `i8` | æŽ§åˆ¶æ¨¡å¼é€‰æ‹© |
| `0x700A` | `spd_ref` | `f32` | ç›®æ ‡é€Ÿåº¦ |
| `0x7019` | `mechPos` | `f32` | æœºæ¢°ä½ç½® |
| `0x701B` | `mechVel` | `f32` | æœºæ¢°é€Ÿåº¦ |
| `0x701C` | `VBUS` | `f32` | æ¯çº¿ç”µåŽ‹ |

## 5ï¼‰å‚æ•°è¯»å†™

è¯»å‚æ•°ï¼š

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode read-param --param-id 0x7019
```

å†™å‚æ•°ï¼š

```bash
motor_cli \
  --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFD \
  --mode write-param --param-id 0x700A --param-value 0.3
```

Python binding ç¤ºä¾‹ï¼š

```python
from motorbridge import Controller

with Controller("can0") as ctrl:
    m = ctrl.add_robstride_motor(127, 0xFD, "rs-06")
    print(m.robstride_ping())
    print(m.robstride_get_param_f32(0x7019, 500))
    m.robstride_write_param_f32(0x700A, 0.3)
    m.close()
```

## 6ï¼‰åè®®é€šä¿¡ç±»åž‹è¦†ç›–æƒ…å†µ

å½“å‰ `motorbridge` å¯¹ RobStride åè®®é€šä¿¡ç±»åž‹çš„è¦†ç›–ï¼š

- å·²ç›´æŽ¥ä½¿ç”¨ï¼š`0(GET_DEVICE_ID)`ã€`1(OPERATION_CONTROL)`ã€`3(ENABLE)`ã€`4(DISABLE)`ã€`6(SET_ZERO_POSITION)`ã€`7(SET_DEVICE_ID)`ã€`17(READ_PARAMETER)`ã€`18(WRITE_PARAMETER)`ã€`22(SAVE_PARAMETERS)`
- å·²æŽ¥æ”¶è§£æžï¼š`2(OPERATION_STATUS)`ã€`21(FAULT_REPORT)`
- åè®®å¸¸é‡å­˜åœ¨ä½†å°šæœªå½¢æˆé«˜å±‚ APIï¼š`23(SET_BAUDRATE)`ã€`24(ACTIVE_REPORT)`ã€`25(SET_PROTOCOL)`

## 7ï¼‰å®Œå–„ç©ºé—´ï¼ˆå·®è·æ€»ç»“ï¼‰

å½“å‰çŠ¶æ€ï¼šæ ¸å¿ƒé—­çŽ¯å·²å¯ç”¨ï¼ˆ`scan/ping/mit/pos-vel/vel/è¯»å†™å‚æ•°/æ”¹ID/è®¾é›¶/å­˜å‚`ï¼‰ã€‚

å½“å‰å·²çŸ¥é—®é¢˜ï¼ˆå®žæµ‹ï¼‰ï¼š

1. `pos-vel` å‚æ•°ç”Ÿæ•ˆæ€§åœ¨éƒ¨åˆ†å›ºä»¶ä¸Šä¸ç¨³å®šï¼š
   - `--vlim`ï¼ˆ`0x7017`ï¼‰å’Œ `--kp`/`loc_kp`ï¼ˆ`0x701E`ï¼‰å¯èƒ½å›žè¯»æ­£å¸¸ï¼Œä½†ä½“æ„Ÿæ•ˆæžœå¼±æˆ–ä¸æ˜Žæ˜¾ã€‚
   - å½“å‰ `MIT` è·¯å¾„ç›¸å¯¹æ›´ç¨³å®šã€‚
2. RobStride é›¶ç‚¹æ ¡å‡†ä»æœªç¨³å®šï¼š
   - å®žéªŒæ€§ `zero` æ—¶åºå¯èƒ½å‘é€/ACK æ­£å¸¸ï¼Œä½†è®¾å¤‡ä¾§ `zero_sta`/`mechPos` æ ¡éªŒä»å¯èƒ½å¤±è´¥ã€‚
   - åœ¨å®Œæˆå›ºä»¶çº§æ—¶åºå®Œå…¨å¯¹é½å‰ï¼Œé›¶ç‚¹æ ¡å‡†è§†ä¸ºæœªå½»åº•è§£å†³ã€‚

å¯ä¼˜å…ˆå¢žå¼ºï¼š

1. CLI å¢žåŠ æ›´è¯­ä¹‰åŒ–çš„ `current/torque` å¿«æ·å‘½ä»¤ï¼ˆå½“å‰å¯ç”¨å†™å‚æ•°å®žçŽ°ï¼Œä½†ä¸ç›´è§‚ï¼‰ã€‚
2. CLI æ‰«ææ”¯æŒå¤š feedback-host å€™é€‰ã€‚
3. æš´éœ² `SET_BAUDRATE / ACTIVE_REPORT / SET_PROTOCOL` çš„é«˜å±‚ APIã€‚
4. `FAULT_REPORT` ç‹¬ç«‹ç»“æž„åŒ–è§£ç è¾“å‡ºã€‚

## 8ï¼‰WS ç½‘å…³ JSON ç¤ºä¾‹

```json
{"op":"set_target","vendor":"robstride","channel":"can0","model":"rs-06","motor_id":127,"feedback_id":255}
{"op":"robstride_ping","timeout_ms":200}
{"op":"robstride_read_param","param_id":28697,"type":"f32","timeout_ms":200}
{"op":"robstride_write_param","param_id":28682,"type":"f32","value":0.3,"verify":true}
{"op":"vel","vel":0.3,"continuous":true}
{"op":"mit","pos":0.0,"vel":0.0,"kp":0.5,"kd":0.2,"tau":0.0,"continuous":true}
{"op":"scan","vendor":"robstride","start_id":1,"end_id":255,"feedback_ids":"0xFD,0xFF,0xFE","timeout_ms":120}
```

## 9ï¼‰å®‰å…¨å»ºè®®

- å…ˆå°é€Ÿåº¦ã€å°å¾ªçŽ¯éªŒè¯ï¼Œå†é€æ­¥å¢žå¤§ã€‚
- åŽ‹æµ‹å‰å…ˆç¡®è®¤ CAN æŽ¥çº¿ã€ç»ˆç«¯ç”µé˜»å’ŒæŽ¥å£çŠ¶æ€ã€‚
- é•¿æ—¶é—´æŽ§åˆ¶å‰å…ˆåš ping/è¯»å‚éªŒè¯ã€‚
- å§‹ç»ˆä¿ç•™æ€¥åœè·¯å¾„ã€‚





