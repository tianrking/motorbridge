use motor_core::bus::{CanBus, CanFrame};
use motor_core::device::MotorDevice;
use motor_core::error::{MotorError, Result};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const Q21: f32 = (1u32 << 21) as f32;

fn rev_to_q21(v: f32) -> i32 {
    (v * Q21).round() as i32
}

fn q21_to_rev(v: i32) -> f32 {
    (v as f32) / Q21
}

#[derive(Debug, Clone, Copy)]
pub struct HexfellowStatus {
    pub mode_display: i8,
    pub statusword: u16,
    pub position_rev: f32,
    pub velocity_rev_s: f32,
    pub torque_permille: i16,
    pub heartbeat_state: Option<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct PosVelTarget {
    pub position_rev: f32,
    pub velocity_rev_s: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct MitTarget {
    pub position_rev: f32,
    pub velocity_rev_s: f32,
    pub torque_nm: f32,
    pub kp: u16,
    pub kd: u16,
    pub limit_permille: u16,
}

pub struct HexfellowMotor {
    pub motor_id: u16,
    pub feedback_id: u16,
    pub model: String,
    bus: Arc<dyn CanBus>,
    sdo_reply_queue: Mutex<VecDeque<[u8; 8]>>,
    heartbeat_state: Mutex<Option<u8>>,
}

impl HexfellowMotor {
    pub fn new(motor_id: u16, feedback_id: u16, model: &str, bus: Arc<dyn CanBus>) -> Result<Self> {
        if model.trim().is_empty() {
            return Err(MotorError::InvalidArgument(
                "hexfellow model cannot be empty".to_string(),
            ));
        }
        if motor_id == 0 || motor_id > 127 {
            return Err(MotorError::InvalidArgument(format!(
                "invalid Hexfellow motor-id {motor_id}, expected 1..127"
            )));
        }
        Ok(Self {
            motor_id,
            feedback_id,
            model: model.to_string(),
            bus,
            sdo_reply_queue: Mutex::new(VecDeque::new()),
            heartbeat_state: Mutex::new(None),
        })
    }

    fn sdo_req_id(&self) -> u32 {
        0x600 + u32::from(self.motor_id)
    }

    fn sdo_rsp_id(&self) -> u32 {
        0x580 + u32::from(self.motor_id)
    }

    fn heartbeat_id(&self) -> u32 {
        0x700 + u32::from(self.motor_id)
    }

    fn tpdo1_id(&self) -> u32 {
        0x180 + u32::from(self.motor_id)
    }

    fn tpdo2_id(&self) -> u32 {
        0x280 + u32::from(self.motor_id)
    }

    fn send_std_frame(&self, arbitration_id: u32, payload: &[u8]) -> Result<()> {
        if payload.len() > 8 {
            return Err(MotorError::InvalidArgument(format!(
                "payload too long: {}, expected <=8",
                payload.len()
            )));
        }
        let mut data = [0u8; 8];
        data[..payload.len()].copy_from_slice(payload);
        self.bus.send(CanFrame {
            arbitration_id,
            data,
            dlc: payload.len() as u8,
            is_extended: false,
            is_rx: false,
        })
    }

    fn send_nmt(&self, command: u8, node_id: u8) -> Result<()> {
        self.send_std_frame(0x000, &[command, node_id])
    }

    fn build_sdo_upload(index: u16, subindex: u8) -> [u8; 8] {
        [
            0x40,
            (index & 0xFF) as u8,
            ((index >> 8) & 0xFF) as u8,
            subindex,
            0,
            0,
            0,
            0,
        ]
    }

    fn build_sdo_download(index: u16, subindex: u8, payload4: [u8; 4], nbytes: usize) -> [u8; 8] {
        let cmd = match nbytes {
            1 => 0x2F,
            2 => 0x2B,
            4 => 0x23,
            _ => 0x23,
        };
        [
            cmd,
            (index & 0xFF) as u8,
            ((index >> 8) & 0xFF) as u8,
            subindex,
            payload4[0],
            payload4[1],
            payload4[2],
            payload4[3],
        ]
    }

    fn pop_matching_sdo_reply(
        &self,
        index: u16,
        subindex: u8,
        timeout: Duration,
    ) -> Result<[u8; 8]> {
        let deadline = Instant::now() + timeout;
        loop {
            let mut pending = self
                .sdo_reply_queue
                .lock()
                .map_err(|_| MotorError::Io("sdo queue lock poisoned".to_string()))?;
            let mut hold = VecDeque::new();
            let mut found = None;
            while let Some(msg) = pending.pop_front() {
                let idx = u16::from(msg[1]) | (u16::from(msg[2]) << 8);
                let sub = msg[3];
                if idx == index && sub == subindex {
                    found = Some(msg);
                    break;
                }
                hold.push_back(msg);
            }
            while let Some(m) = hold.pop_front() {
                pending.push_back(m);
            }
            drop(pending);
            if let Some(msg) = found {
                return Ok(msg);
            }
            if Instant::now() >= deadline {
                return Err(MotorError::Timeout(format!(
                    "sdo response timeout idx=0x{index:04X} sub=0x{subindex:02X}"
                )));
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn sdo_read_raw(&self, index: u16, subindex: u8, timeout: Duration) -> Result<[u8; 4]> {
        let req = Self::build_sdo_upload(index, subindex);
        self.send_std_frame(self.sdo_req_id(), &req)?;
        let rsp = self.pop_matching_sdo_reply(index, subindex, timeout)?;
        let cmd = rsp[0];
        if cmd == 0x80 {
            let abort = u32::from_le_bytes([rsp[4], rsp[5], rsp[6], rsp[7]]);
            return Err(MotorError::Protocol(format!(
                "sdo abort idx=0x{index:04X} sub=0x{subindex:02X} code=0x{abort:08X}"
            )));
        }
        if !matches!(cmd, 0x43 | 0x4B | 0x4F | 0x47) {
            return Err(MotorError::Protocol(format!(
                "unexpected sdo read cmd=0x{cmd:02X} idx=0x{index:04X} sub=0x{subindex:02X}"
            )));
        }
        Ok([rsp[4], rsp[5], rsp[6], rsp[7]])
    }

    fn sdo_write_raw(
        &self,
        index: u16,
        subindex: u8,
        payload4: [u8; 4],
        nbytes: usize,
        timeout: Duration,
    ) -> Result<()> {
        let req = Self::build_sdo_download(index, subindex, payload4, nbytes);
        self.send_std_frame(self.sdo_req_id(), &req)?;
        let rsp = self.pop_matching_sdo_reply(index, subindex, timeout)?;
        let cmd = rsp[0];
        if cmd == 0x80 {
            let abort = u32::from_le_bytes([rsp[4], rsp[5], rsp[6], rsp[7]]);
            return Err(MotorError::Protocol(format!(
                "sdo abort idx=0x{index:04X} sub=0x{subindex:02X} code=0x{abort:08X}"
            )));
        }
        if cmd != 0x60 {
            return Err(MotorError::Protocol(format!(
                "unexpected sdo write ack cmd=0x{cmd:02X} idx=0x{index:04X} sub=0x{subindex:02X}"
            )));
        }
        Ok(())
    }

    fn sdo_read_i32(&self, index: u16, subindex: u8, timeout: Duration) -> Result<i32> {
        Ok(i32::from_le_bytes(
            self.sdo_read_raw(index, subindex, timeout)?,
        ))
    }

    fn sdo_read_u16(&self, index: u16, subindex: u8, timeout: Duration) -> Result<u16> {
        let raw = self.sdo_read_raw(index, subindex, timeout)?;
        Ok(u16::from_le_bytes([raw[0], raw[1]]))
    }

    fn sdo_read_i16(&self, index: u16, subindex: u8, timeout: Duration) -> Result<i16> {
        let raw = self.sdo_read_raw(index, subindex, timeout)?;
        Ok(i16::from_le_bytes([raw[0], raw[1]]))
    }

    fn sdo_read_i8(&self, index: u16, subindex: u8, timeout: Duration) -> Result<i8> {
        let raw = self.sdo_read_raw(index, subindex, timeout)?;
        Ok(raw[0] as i8)
    }

    fn sdo_write_i8(&self, index: u16, subindex: u8, value: i8, timeout: Duration) -> Result<()> {
        self.sdo_write_raw(index, subindex, [value as u8, 0, 0, 0], 1, timeout)
    }

    fn sdo_write_u16(&self, index: u16, subindex: u8, value: u16, timeout: Duration) -> Result<()> {
        let b = value.to_le_bytes();
        self.sdo_write_raw(index, subindex, [b[0], b[1], 0, 0], 2, timeout)
    }

    fn sdo_write_u32(&self, index: u16, subindex: u8, value: u32, timeout: Duration) -> Result<()> {
        self.sdo_write_raw(index, subindex, value.to_le_bytes(), 4, timeout)
    }

    fn sdo_write_i32(&self, index: u16, subindex: u8, value: i32, timeout: Duration) -> Result<()> {
        self.sdo_write_raw(index, subindex, value.to_le_bytes(), 4, timeout)
    }

    pub fn ensure_mode_enabled(&self, mode: i8, timeout: Duration) -> Result<()> {
        self.send_nmt(0x01, self.motor_id as u8)?;
        std::thread::sleep(Duration::from_millis(20));
        self.sdo_write_i8(0x6060, 0x00, mode, timeout)?;
        for cw in [0x0006u16, 0x0007u16, 0x000Fu16] {
            self.sdo_write_u16(0x6040, 0x00, cw, timeout)?;
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    pub fn enable_drive(&self, timeout: Duration) -> Result<()> {
        self.ensure_mode_enabled(1, timeout)
    }

    pub fn disable_drive(&self, timeout: Duration) -> Result<()> {
        self.sdo_write_u16(0x6040, 0x00, 0x0006, timeout)
    }

    pub fn command_pos_vel(&self, target: PosVelTarget, timeout: Duration) -> Result<()> {
        self.ensure_mode_enabled(1, timeout)?;
        let pos_q21 = rev_to_q21(target.position_rev);
        let vel_q21 = rev_to_q21(target.velocity_rev_s.abs());
        self.sdo_write_u32(0x6081, 0x00, vel_q21 as u32, timeout)?;
        self.sdo_write_i32(0x607A, 0x00, pos_q21, timeout)?;
        self.sdo_write_u16(0x6040, 0x00, 0x002F, timeout)?;
        self.sdo_write_u16(0x6040, 0x00, 0x003F, timeout)?;
        self.sdo_write_u16(0x6040, 0x00, 0x002F, timeout)?;
        Ok(())
    }

    pub fn command_mit(&self, target: MitTarget, timeout: Duration) -> Result<()> {
        self.ensure_mode_enabled(5, timeout)?;
        self.sdo_write_i32(0x2003, 0x01, rev_to_q21(target.position_rev), timeout)?;
        self.sdo_write_i32(0x2003, 0x02, rev_to_q21(target.velocity_rev_s), timeout)?;
        self.sdo_write_i32(0x2003, 0x03, rev_to_q21(target.torque_nm), timeout)?;
        self.sdo_write_u16(0x2003, 0x04, target.kp, timeout)?;
        self.sdo_write_u16(0x2003, 0x05, target.kd, timeout)?;
        self.sdo_write_u16(0x2003, 0x06, target.limit_permille, timeout)?;
        Ok(())
    }

    pub fn query_status(&self, timeout: Duration) -> Result<HexfellowStatus> {
        let mode_display = self.sdo_read_i8(0x6061, 0x00, timeout)?;
        let statusword = self.sdo_read_u16(0x6041, 0x00, timeout)?;
        let pos_q21 = self.sdo_read_i32(0x6064, 0x00, timeout)?;
        let vel_q21 = self.sdo_read_i32(0x606C, 0x00, timeout)?;
        let tau_permille = self.sdo_read_i16(0x6077, 0x00, timeout).unwrap_or(0);
        let hb = self
            .heartbeat_state
            .lock()
            .map_err(|_| MotorError::Io("heartbeat lock poisoned".to_string()))?
            .to_owned();
        Ok(HexfellowStatus {
            mode_display,
            statusword,
            position_rev: q21_to_rev(pos_q21),
            velocity_rev_s: q21_to_rev(vel_q21),
            torque_permille: tau_permille,
            heartbeat_state: hb,
        })
    }
}

impl MotorDevice for HexfellowMotor {
    fn vendor(&self) -> &'static str {
        "hexfellow"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn motor_id(&self) -> u16 {
        self.motor_id
    }

    fn feedback_id(&self) -> u16 {
        self.feedback_id
    }

    fn enable(&self) -> Result<()> {
        self.enable_drive(Duration::from_millis(200))
    }

    fn disable(&self) -> Result<()> {
        self.disable_drive(Duration::from_millis(200))
    }

    fn accepts_frame(&self, frame: &CanFrame) -> bool {
        !frame.is_extended
            && (frame.arbitration_id == self.sdo_rsp_id()
                || frame.arbitration_id == self.heartbeat_id()
                || frame.arbitration_id == self.tpdo1_id()
                || frame.arbitration_id == self.tpdo2_id())
    }

    fn process_feedback_frame(&self, frame: CanFrame) -> Result<()> {
        if frame.arbitration_id == self.sdo_rsp_id() {
            self.sdo_reply_queue
                .lock()
                .map_err(|_| MotorError::Io("sdo queue lock poisoned".to_string()))?
                .push_back(frame.data);
            return Ok(());
        }
        if frame.arbitration_id == self.heartbeat_id() && frame.dlc >= 1 {
            self.heartbeat_state
                .lock()
                .map_err(|_| MotorError::Io("heartbeat lock poisoned".to_string()))?
                .replace(frame.data[0]);
        }
        Ok(())
    }
}
