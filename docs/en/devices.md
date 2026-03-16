# Supported Devices

## Production Support

| Brand | Models | Control Modes | Register R/W | ABI Coverage | Notes |
|---|---|---|---|---|---|
| Damiao | 3507, 4310, 4310P, 4340, 4340P, 6006, 8006, 8009, 10010L, 10010, H3510, G6215, H6220, JH11, 6248P | MIT, POS_VEL, VEL, FORCE_POS | Yes (f32/u32) | Yes | Run per-model hardware regression |

## Template (Not Production)

| Brand | Models | Control Modes | Register R/W | ABI Coverage | Notes |
|---|---|---|---|---|---|
| template_vendor | model_a (placeholder) | Placeholder only | Placeholder only | No | Scaffolding for new vendor integration |

## Mode Legend

- MIT: position + velocity + stiffness + damping + torque feedforward
- POS_VEL: position + velocity limit
- VEL: velocity control
- FORCE_POS: position + velocity limit + torque ratio
