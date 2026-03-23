use crate::bus::{CanBus, CanFrame};
use crate::error::{MotorError, Result};
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const TX_FRAME_LEN: usize = 30;
const RX_FRAME_LEN: usize = 16;

struct Inner {
    port: Box<dyn SerialPort>,
    rx_buf: VecDeque<u8>,
}

pub struct DmSerialBus {
    inner: Mutex<Inner>,
}

impl DmSerialBus {
    pub fn open(port: &str, baud: u32) -> Result<Self> {
        let port_obj = serialport::new(port, baud)
            .timeout(Duration::from_millis(2))
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .flow_control(FlowControl::None)
            .open()
            .map_err(|e| MotorError::Io(format!("open serial port {port} failed: {e}")))?;
        Ok(Self {
            inner: Mutex::new(Inner {
                port: port_obj,
                rx_buf: VecDeque::with_capacity(1024),
            }),
        })
    }

    fn encode_tx(frame: CanFrame) -> Result<[u8; TX_FRAME_LEN]> {
        if frame.dlc > 8 {
            return Err(MotorError::InvalidArgument(format!(
                "invalid DLC {}, expected <= 8",
                frame.dlc
            )));
        }
        if !frame.is_extended && frame.arbitration_id > 0x7FF {
            return Err(MotorError::InvalidArgument(format!(
                "invalid arbitration_id {:X}, expected 11-bit std id",
                frame.arbitration_id
            )));
        }

        let mut out = [0u8; TX_FRAME_LEN];
        out[0] = 0x55;
        out[1] = 0xAA;
        out[2] = 0x1E;
        out[3] = 0x03; // non-feedback CAN forwarding
        out[4..8].copy_from_slice(&1u32.to_le_bytes()); // sendTimes
        out[8..12].copy_from_slice(&10u32.to_le_bytes()); // timeInterval
        out[12] = u8::from(frame.is_extended); // IDType
        out[13..17].copy_from_slice(&frame.arbitration_id.to_le_bytes()); // canId
        out[17] = 0; // frameType: data frame
        out[18] = frame.dlc;
        out[19] = 0; // idAcc
        out[20] = 0; // dataAcc
        out[21..29].copy_from_slice(&frame.data);
        out[29] = 0; // crc (ignored by adapter in the reference implementation)
        Ok(out)
    }

    fn try_parse_rx(buf: &mut VecDeque<u8>) -> Option<CanFrame> {
        while let Some(&first) = buf.front() {
            if first == 0xAA {
                break;
            }
            let _ = buf.pop_front();
        }
        while buf.len() >= RX_FRAME_LEN {
            if buf.front().copied() != Some(0xAA) {
                let _ = buf.pop_front();
                continue;
            }
            let mut raw = [0u8; RX_FRAME_LEN];
            for (i, b) in buf.iter().take(RX_FRAME_LEN).enumerate() {
                raw[i] = *b;
            }
            for _ in 0..RX_FRAME_LEN {
                let _ = buf.pop_front();
            }

            if raw[15] != 0x55 {
                continue;
            }
            if raw[1] != 0x11 {
                continue;
            }
            let flags = raw[2];
            let dlc = flags & 0x3F;
            let is_extended = (flags & 0x40) != 0;
            let is_rtr = (flags & 0x80) != 0;
            if is_rtr {
                continue;
            }
            let arbitration_id = u32::from_le_bytes([raw[3], raw[4], raw[5], raw[6]]);
            let mut data = [0u8; 8];
            data.copy_from_slice(&raw[7..15]);
            return Some(CanFrame {
                arbitration_id,
                data,
                dlc,
                is_extended,
                is_rx: true,
            });
        }
        None
    }
}

impl CanBus for DmSerialBus {
    fn send(&self, frame: CanFrame) -> Result<()> {
        let raw = Self::encode_tx(frame)?;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| MotorError::Io("dm-serial lock poisoned".to_string()))?;
        inner
            .port
            .write_all(&raw)
            .map_err(|e| MotorError::Io(format!("dm-serial write failed: {e}")))?;
        Ok(())
    }

    fn recv(&self, timeout: Duration) -> Result<Option<CanFrame>> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(3600));
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| MotorError::Io("dm-serial lock poisoned".to_string()))?;

        loop {
            if let Some(frame) = Self::try_parse_rx(&mut inner.rx_buf) {
                return Ok(Some(frame));
            }

            let mut tmp = [0u8; 256];
            match inner.port.read(&mut tmp) {
                Ok(n) if n > 0 => {
                    inner.rx_buf.extend(tmp[..n].iter().copied());
                }
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(MotorError::Io(format!("dm-serial read failed: {e}"))),
            }

            if Instant::now() >= deadline {
                return Ok(None);
            }
        }
    }

    fn shutdown(&self) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| MotorError::Io("dm-serial lock poisoned".to_string()))?;
        inner
            .port
            .flush()
            .map_err(|e| MotorError::Io(format!("dm-serial flush failed: {e}")))?;
        Ok(())
    }
}
