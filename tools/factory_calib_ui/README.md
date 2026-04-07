# factory_calib_ui

A lightweight web-based factory console for motor ID calibration.

Current support:
- Damiao
- RobStride (scan + set-id + verify)
- MyActuator (scan + verify)
- HighTorque (scan + verify)
- Hexfellow (scan + verify)

- Backend: `server.py` (Python stdlib only, no extra dependencies)
- Calibration engine: uses `motor_cli` (`--mode scan`, `--set-motor-id`, `--set-feedback-id`, `--verify-id`)
- Frontend: `index.html` (scan table, one-click set-id, verify, logs)

## Start

From repo root:

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
cargo build -p motor_cli --release
python3 tools/factory_calib_ui/server.py --bind 0.0.0.0 --port 18100
```

Open:

- local: `http://127.0.0.1:18100`
- LAN: `http://<your-ip>:18100`

## Workflow

1. Configure `channel`, then set per-vendor rows (`Scan` checkbox, `model`, `start_id`, `end_id`, and vendor extra fields)
2. Click `Scan`
3. Fill `New ESC_ID / New MST_ID` per row
4. Click `Set ID` (auto `store + verify`)
5. Optionally click `Verify`

Notes:
- Each vendor has independent scan range and model config.
- Uncheck a vendor row to skip scanning that vendor.
- Vendors are multi-select and can be scanned together.
- `Set ID` is currently supported for Damiao and RobStride.
- For RobStride rows, `New MST_ID` is display-only (ID write applies to device ID).

## Notes

- For safety, calibrate one motor at a time when rewriting IDs.
- Linux SocketCAN channel should be plain `can0` (not `can0@1000000`).
- Current UI focuses on factory ID calibration workflows.


Channel troubleshooting reference: see docs/en/can_debugging.md (covers can0 and slcan0 setup).
