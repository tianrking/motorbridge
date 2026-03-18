#![cfg(target_os = "linux")]

use crate::bus::{CanBus, CanFrame};
use crate::error::{MotorError, Result};
use core::ffi::{c_char, c_int, c_short, c_uint, c_void};
use std::ffi::CString;
use std::mem::size_of;
use std::os::fd::RawFd;
use std::sync::Mutex;
use std::time::Duration;

const AF_CAN: c_int = 29;
const PF_CAN: c_int = AF_CAN;
const SOCK_RAW: c_int = 3;
const CAN_RAW: c_int = 1;
const POLLIN: c_short = 0x0001;
const CAN_EFF_FLAG: u32 = 0x8000_0000;
const CAN_EFF_MASK: u32 = 0x1FFF_FFFF;
const CAN_SFF_MASK: u32 = 0x0000_07FF;

#[repr(C)]
struct SockAddrCan {
    can_family: u16,
    can_ifindex: c_int,
    can_addr: [u8; 8],
}

#[repr(C)]
struct PollFd {
    fd: c_int,
    events: c_short,
    revents: c_short,
}

#[repr(C)]
struct CanFrameRaw {
    can_id: u32,
    can_dlc: u8,
    __pad: u8,
    __res0: u8,
    __res1: u8,
    data: [u8; 8],
}

unsafe extern "C" {
    fn socket(domain: c_int, typ: c_int, protocol: c_int) -> c_int;
    fn bind(sockfd: c_int, addr: *const c_void, addrlen: u32) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn poll(fds: *mut PollFd, nfds: c_uint, timeout: c_int) -> c_int;
    fn if_nametoindex(ifname: *const c_char) -> c_uint;
}

fn last_os_error(prefix: &str) -> MotorError {
    let err = std::io::Error::last_os_error();
    let hint = match err.raw_os_error() {
        Some(100) => " (hint: can interface is down; run `ip -details link show can0` then bring it up)",
        Some(6) => " (hint: can device/address is unavailable; check USB-CAN adapter and `ip link show`)",
        Some(19) => " (hint: interface not found; verify channel name like can0 and adapter connection)",
        _ => "",
    };
    MotorError::Io(format!("{prefix}: {err}{hint}"))
}

pub struct SocketCanBus {
    fd: Mutex<Option<RawFd>>,
}

impl SocketCanBus {
    pub fn open(interface: &str) -> Result<Self> {
        let iface = CString::new(interface)
            .map_err(|_| MotorError::InvalidArgument("interface contains NUL".to_string()))?;

        let index = unsafe { if_nametoindex(iface.as_ptr()) };
        if index == 0 {
            return Err(last_os_error(&format!(
                "if_nametoindex failed for {interface}"
            )));
        }

        let fd = unsafe { socket(PF_CAN, SOCK_RAW, CAN_RAW) };
        if fd < 0 {
            return Err(last_os_error("socket(PF_CAN, SOCK_RAW, CAN_RAW) failed"));
        }

        let addr = SockAddrCan {
            can_family: AF_CAN as u16,
            can_ifindex: index as c_int,
            can_addr: [0u8; 8],
        };

        let bind_rc = unsafe {
            bind(
                fd,
                (&addr as *const SockAddrCan).cast::<c_void>(),
                size_of::<SockAddrCan>() as u32,
            )
        };
        if bind_rc < 0 {
            let _ = unsafe { close(fd) };
            return Err(last_os_error(&format!("bind on {interface} failed")));
        }

        Ok(Self {
            fd: Mutex::new(Some(fd)),
        })
    }

    fn with_fd<T>(&self, f: impl FnOnce(RawFd) -> Result<T>) -> Result<T> {
        let guard = self
            .fd
            .lock()
            .map_err(|_| MotorError::Io("socket fd lock poisoned".to_string()))?;
        let fd = guard
            .as_ref()
            .copied()
            .ok_or_else(|| MotorError::Io("socket already closed".to_string()))?;
        f(fd)
    }
}

impl CanBus for SocketCanBus {
    fn send(&self, frame: CanFrame) -> Result<()> {
        self.with_fd(|fd| {
            if !frame.is_extended && frame.arbitration_id > CAN_SFF_MASK {
                return Err(MotorError::InvalidArgument(format!(
                    "invalid arbitration_id {:X}, expected 11-bit std id",
                    frame.arbitration_id
                )));
            }
            if frame.is_extended && frame.arbitration_id > CAN_EFF_MASK {
                return Err(MotorError::InvalidArgument(format!(
                    "invalid arbitration_id {:X}, expected 29-bit ext id",
                    frame.arbitration_id
                )));
            }
            if frame.dlc > 8 {
                return Err(MotorError::InvalidArgument(format!(
                    "invalid DLC {}, expected <= 8",
                    frame.dlc
                )));
            }

            let raw = CanFrameRaw {
                can_id: if frame.is_extended {
                    frame.arbitration_id | CAN_EFF_FLAG
                } else {
                    frame.arbitration_id
                },
                can_dlc: frame.dlc,
                __pad: 0,
                __res0: 0,
                __res1: 0,
                data: frame.data,
            };

            let n = unsafe {
                write(
                    fd,
                    (&raw as *const CanFrameRaw).cast::<c_void>(),
                    size_of::<CanFrameRaw>(),
                )
            };
            if n != size_of::<CanFrameRaw>() as isize {
                return Err(last_os_error("socketcan write failed"));
            }
            Ok(())
        })
    }

    fn recv(&self, timeout: Duration) -> Result<Option<CanFrame>> {
        self.with_fd(|fd| {
            let timeout_ms = if timeout.is_zero() {
                0
            } else {
                timeout.as_millis().min(c_int::MAX as u128) as c_int
            };

            let mut pfd = PollFd {
                fd,
                events: POLLIN,
                revents: 0,
            };

            let rc = unsafe { poll(&mut pfd as *mut PollFd, 1, timeout_ms) };
            if rc < 0 {
                return Err(last_os_error("poll failed"));
            }
            if rc == 0 {
                return Ok(None);
            }

            let mut raw = CanFrameRaw {
                can_id: 0,
                can_dlc: 0,
                __pad: 0,
                __res0: 0,
                __res1: 0,
                data: [0u8; 8],
            };

            let n = unsafe {
                read(
                    fd,
                    (&mut raw as *mut CanFrameRaw).cast::<c_void>(),
                    size_of::<CanFrameRaw>(),
                )
            };
            if n != size_of::<CanFrameRaw>() as isize {
                return Err(last_os_error("socketcan read failed"));
            }

            Ok(Some(CanFrame {
                arbitration_id: if (raw.can_id & CAN_EFF_FLAG) != 0 {
                    raw.can_id & CAN_EFF_MASK
                } else {
                    raw.can_id & CAN_SFF_MASK
                },
                data: raw.data,
                dlc: raw.can_dlc.min(8),
                is_extended: (raw.can_id & CAN_EFF_FLAG) != 0,
                is_rx: true,
            }))
        })
    }

    fn shutdown(&self) -> Result<()> {
        let mut guard = self
            .fd
            .lock()
            .map_err(|_| MotorError::Io("socket fd lock poisoned".to_string()))?;
        if let Some(fd) = guard.take() {
            let rc = unsafe { close(fd) };
            if rc < 0 {
                return Err(last_os_error("close socketcan fd failed"));
            }
        }
        Ok(())
    }
}

impl Drop for SocketCanBus {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.fd.lock() {
            if let Some(fd) = guard.take() {
                let _ = unsafe { close(fd) };
            }
        }
    }
}
