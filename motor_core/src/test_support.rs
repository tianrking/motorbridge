use crate::bus::{CanBus, CanFrame};
use crate::error::{MotorError, Result};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

pub struct MockBus {
    pub rx: Mutex<VecDeque<CanFrame>>,
    pub sent: Mutex<Vec<CanFrame>>,
    pub shutdown_count: AtomicUsize,
    pub fail_recv: AtomicBool,
}

impl MockBus {
    pub fn new() -> Self {
        Self {
            rx: Mutex::new(VecDeque::new()),
            sent: Mutex::new(Vec::new()),
            shutdown_count: AtomicUsize::new(0),
            fail_recv: AtomicBool::new(false),
        }
    }

    pub fn push_rx(&self, frame: CanFrame) {
        if let Ok(mut rx) = self.rx.lock() {
            rx.push_back(frame);
        }
    }

    pub fn set_fail_recv(&self, fail: bool) {
        self.fail_recv.store(fail, Ordering::SeqCst);
    }
}

impl Default for MockBus {
    fn default() -> Self {
        Self::new()
    }
}

impl CanBus for MockBus {
    fn send(&self, frame: CanFrame) -> Result<()> {
        self.sent
            .lock()
            .map_err(|_| MotorError::Io("sent lock poisoned".to_string()))?
            .push(frame);
        Ok(())
    }

    fn recv(&self, _timeout: Duration) -> Result<Option<CanFrame>> {
        if self.fail_recv.load(Ordering::SeqCst) {
            return Err(MotorError::Io("injected recv error".to_string()));
        }
        Ok(self
            .rx
            .lock()
            .map_err(|_| MotorError::Io("rx lock poisoned".to_string()))?
            .pop_front())
    }

    fn shutdown(&self) -> Result<()> {
        self.shutdown_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
