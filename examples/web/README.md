# Web HMI Examples

CAN bring-up and troubleshooting guide: [../../docs/en/can_debugging.md](../../docs/en/can_debugging.md)

## ws_quad_sync_hmi.html

Single-slider Web HMI for synchronized 4-motor angle control via `ws_gateway`.

Default targets:

- Damiao `0x01` (`4340P`, feedback `0x11`)
- Damiao `0x07` (`4310`, feedback `0x17`)
- MyActuator `1` (`X8`, feedback `0x241`)
- HighTorque `1` (`hightorque`, feedback `0x01`)

### Run

```bash
# terminal A: start gateway
cargo run -p ws_gateway --release -- --bind 0.0.0.0:9002 --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 20

# terminal B: serve static files from repo root
python3 -m http.server 18080

# browser:
http://127.0.0.1:18080/examples/web/ws_quad_sync_hmi.html
```

### Notes

- The page creates one WS session per motor for better sync and lower target-switch overhead.
- Position commands now default to `continuous=true`, so one send will keep holding position; use "Stop All" or "Disable All" to release.
- Slider value is angle in radians and is sent to all connected motors.
- If you see occasional disconnects on one motor session:
  - raise gateway `--dt-ms` from `20` to `50`
  - keep HMI `发送周期(ms)` around `60~100`
  - keep `错峰(ms)` around `8~20`

## ws_quad_sync_hmi_rs.html

Based on `ws_quad_sync_hmi.html`, this variant only replaces motor #4 from HighTorque to RobStride. The other three targets stay the same.

Default targets:

- Damiao `0x01` (`4340P`, feedback `0x11`)
- Damiao `0x07` (`4310`, feedback `0x17`)
- MyActuator `1` (`X8`, feedback `0x241`)
- RobStride `127` (`rs-00`, feedback `0xFE`)

### Run

```bash
# terminal A: start gateway
cargo run -p ws_gateway --release -- --bind 0.0.0.0:9002 --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 20

# terminal B: serve static files from repo root
python3 -m http.server 18080

# browser (RobStride variant):
http://127.0.0.1:18080/examples/web/ws_quad_sync_hmi_rs.html
```
