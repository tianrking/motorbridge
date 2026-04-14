# motor_vendor_hightorque

HighTorque vendor crate aligned with the same module layout used by other vendors.

## Layout

- `src/controller.rs`: bus lifecycle and motor factory (`HightorqueController`)
- `src/motor.rs`: control and state APIs (`HightorqueMotor`)
- `src/protocol.rs`: frame decode and unit conversion helpers
- `src/lib.rs`: module wiring and public re-exports

## Notes

- Current HighTorque path is direct CAN native flow (ht_can style frames).
- Unified API compatibility:
  - supported: `mit`, `pos_vel`, `vel`, `force_pos`
  - `pos_vel` and `force_pos` map to native pos+vel+tqe
