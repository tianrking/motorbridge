# æ”¯æŒè®¾å¤‡

<!-- channel-compat-note -->
## é€šé“å…¼å®¹è¯´æ˜Žï¼ˆPCAN + slcan + Damiao ä¸²å£æ¡¥ï¼‰

- Linux SocketCAN ç›´æŽ¥ä½¿ç”¨ç½‘å¡åï¼š`can0`ã€`can1`ã€`slcan0`ã€‚
- ä¸²å£ç±» USB-CAN éœ€å…ˆåˆ›å»ºå¹¶æ‹‰èµ· `slcan0`ï¼š`sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0 && sudo ip link set slcan0 up`ã€‚
- ä»… Damiao å¯é€‰ä¸²å£æ¡¥é“¾è·¯ï¼š`--transport dm-serial --serial-port /dev/ttyACM0 --serial-baud 921600`ã€‚
- Linux SocketCAN ä¸‹ `--channel` ä¸è¦å¸¦ `@bitrate`ï¼ˆä¾‹å¦‚ `can0@1000000` æ— æ•ˆï¼‰ã€‚
- Windowsï¼ˆPCAN åŽç«¯ï¼‰ä¸­ï¼Œ`can0/can1` æ˜ å°„ `PCAN_USBBUS1/2`ï¼Œå¯é€‰ `@bitrate` åŽç¼€ã€‚


## è®¾å¤‡æ”¯æŒå…¨æ™¯å›¾

```mermaid
mindmap
  root((motorbridge è®¾å¤‡))
    ç”Ÿäº§æ”¯æŒ
      Damiao
        3507
        4310 / 4310P
        4340 / 4340P
        6006 / 8006 / 8009
        10010 / 10010L
        H3510 / G6215 / H6220 / JH11 / 6248P
        æ¨¡å¼
          MIT / POS_VEL / VEL / FORCE_POS
      RobStride
        rs-00 / rs-01 / rs-02
        rs-03 / rs-04 / rs-05 / rs-06
        æ¨¡å¼
          MIT / POS_VEL / VEL / ping / enable-disable / å‚æ•°è¯»å†™ / set-id / zero
      MyActuator
        X ç³»åˆ—ï¼ˆæŒ‰ ID é€šä¿¡ï¼‰
        æ¨¡å¼
          enable / disable / stop / status / current / vel / pos
      HighTorque
        hightorqueï¼ˆåŽŸç”Ÿ ht_can v1.5.5ï¼‰
        æ¨¡å¼
          scan / read / mit / pos-vel / vel / stop
    æ¨¡æ¿
      template_vendor
        model_a
```

## ç”Ÿäº§å¯ç”¨æ”¯æŒ

| å“ç‰Œ | åž‹å· | æŽ§åˆ¶æ¨¡å¼ | å¯„å­˜å™¨è¯»å†™ | ABI è¦†ç›– | è¯´æ˜Ž |
|---|---|---|---|---|---|
| Damiao | 3507, 4310, 4310P, 4340, 4340P, 6006, 8006, 8009, 10010L, 10010, H3510, G6215, H6220, JH11, 6248P | MIT, POS_VEL, VEL, FORCE_POS | æ”¯æŒï¼ˆf32/u32ï¼‰ | æ”¯æŒ | å»ºè®®æŒ‰åž‹å·å®žæœºå›žå½’ |
| RobStride | rs-00, rs-01, rs-02, rs-03, rs-04, rs-05, rs-06 | scan / ping / enable / disable / MIT / POS_VEL / VEL / read-param / write-param / set-id / zero | æ”¯æŒï¼ˆi8/u8/u16/u32/f32ï¼‰ | æ”¯æŒ | ä½¿ç”¨ 29-bit æ‰©å±• CAN IDï¼›é»˜è®¤ `feedback-id=0xFD`ï¼Œå¹¶å›žé€€æŽ¢æµ‹ `0xFF/0xFE` |
| MyActuator | X ç³»åˆ—ï¼ˆè¿è¡Œæ—¶åž‹å·å­—ç¬¦ä¸²ï¼Œé»˜è®¤ `X8`ï¼‰ | enableã€disableã€stopã€statusã€currentã€velã€posã€versionã€mode-query | æš‚æ— ï¼ˆCLI å‘½ä»¤çº§æ”¯æŒï¼‰ | æ”¯æŒ | ä½¿ç”¨æ ‡å‡† 11-bit IDï¼š`0x140+id` / `0x240+id`ï¼›å¸¸ç”¨ ID èŒƒå›´ 1..32 |
| HighTorque | hightorqueï¼ˆè¿è¡Œæ—¶åž‹å·å­—ç¬¦ä¸²ï¼›åŽŸç”Ÿ `ht_can v1.5.5`ï¼‰ | scanã€readã€MITã€POS_VELã€VELã€stopã€brakeã€rezero | æš‚æ— ï¼ˆvendor å‘½ä»¤çº§æ”¯æŒï¼‰ | æ”¯æŒ | å¯¹å¤–ç»Ÿä¸€ `rad/rad/s/Nm`ï¼ŒåŽŸç”Ÿ payload ç¼©æ”¾ç”±å®žçŽ°å±‚å¤„ç† |

## æ¨¡æ¿ï¼ˆéžç”Ÿäº§ï¼‰

| å“ç‰Œ | åž‹å· | æŽ§åˆ¶æ¨¡å¼ | å¯„å­˜å™¨è¯»å†™ | ABI è¦†ç›– | è¯´æ˜Ž |
|---|---|---|---|---|---|
| template_vendor | model_aï¼ˆå ä½ï¼‰ | å ä½å®žçŽ° | å ä½å®žçŽ° | ä¸æ”¯æŒ | ç”¨äºŽæ–°åŽ‚å•†æŽ¥å…¥æ¨¡æ¿ |

## æ¨¡å¼è¯´æ˜Ž

- MITï¼šä½ç½® + é€Ÿåº¦ + åˆšåº¦ + é˜»å°¼ + åŠ›çŸ©å‰é¦ˆ
- POS_VELï¼šä½ç½® + é€Ÿåº¦é™åˆ¶
- VELï¼šé€Ÿåº¦æŽ§åˆ¶
- FORCE_POSï¼šä½ç½® + é€Ÿåº¦é™åˆ¶ + åŠ›çŸ©æ¯”ä¾‹

## Update (2026-04): Damiao / RobStride Capability Matrix

| 品牌 | 核心能力状态 | 已验证能力 | 备注 |
|---|---|---|---|
| Damiao | 生产可用 | scan / enable / disable / MIT / POS_VEL / VEL / FORCE_POS / set-id / set-zero | 改 ID 建议始终使用 `--store 1 --verify-id 1` |
| RobStride | 生产可用 | scan / ping / enable / disable / MIT / POS_VEL / VEL / read-param / write-param / set-id / zero | 默认 `feedback-id=0xFD`，并回退探测 `0xFF/0xFE`；`pos-vel` 忽略 `--vel/--kd/--tau`（warning，不报错） |

