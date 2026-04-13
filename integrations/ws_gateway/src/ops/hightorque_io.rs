use motor_core::bus::{CanBus, CanFrame};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub(crate) struct HighTorqueStatus {
    pub(crate) motor_id: u16,
    pub(crate) pos_raw: i16,
    pub(crate) vel_raw: i16,
    pub(crate) tqe_raw: i16,
}

fn can_ext_id_for_motor(motor_id: u16) -> u32 {
    u32::from(0x8000u16 | motor_id)
}

pub(crate) fn send_hightorque_ext(
    bus: &dyn CanBus,
    motor_id: u16,
    payload: &[u8],
) -> Result<(), String> {
    if payload.len() > 8 {
        return Err("payload too long (max 8 bytes)".to_string());
    }
    let mut data = [0u8; 8];
    data[..payload.len()].copy_from_slice(payload);
    bus.send(CanFrame {
        arbitration_id: can_ext_id_for_motor(motor_id),
        data,
        dlc: payload.len() as u8,
        is_extended: true,
        is_rx: false,
    })
    .map_err(|e| e.to_string())
}

fn decode_hightorque_read_reply(frame: CanFrame) -> Option<HighTorqueStatus> {
    if frame.dlc < 8 || frame.data[0] != 0x27 || frame.data[1] != 0x01 {
        return None;
    }
    let motor_id = if !frame.is_extended && (frame.arbitration_id & 0x00FF) == 0 {
        ((frame.arbitration_id >> 8) & 0x7F) as u16
    } else {
        (frame.arbitration_id & 0x7FF) as u16
    };
    Some(HighTorqueStatus {
        motor_id,
        pos_raw: i16::from_le_bytes([frame.data[2], frame.data[3]]),
        vel_raw: i16::from_le_bytes([frame.data[4], frame.data[5]]),
        tqe_raw: i16::from_le_bytes([frame.data[6], frame.data[7]]),
    })
}

pub(crate) fn wait_hightorque_status_for_motor(
    bus: &dyn CanBus,
    motor_id: u16,
    timeout: Duration,
) -> Result<Option<HighTorqueStatus>, String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let left = deadline.saturating_duration_since(Instant::now());
        if let Some(frame) = bus
            .recv(left.min(Duration::from_millis(20)))
            .map_err(|e| e.to_string())?
        {
            if let Some(status) = decode_hightorque_read_reply(frame) {
                if status.motor_id == motor_id {
                    return Ok(Some(status));
                }
            }
        }
    }
    Ok(None)
}
