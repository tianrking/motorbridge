use crate::bus::CanBus;
use crate::device::MotorDevice;
use crate::error::{MotorError, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct CoreController {
    bus: Arc<dyn CanBus>,
    devices: Arc<Mutex<HashMap<u16, Arc<dyn MotorDevice>>>>,
    polling_active: Arc<AtomicBool>,
    polling_thread: Mutex<Option<JoinHandle<()>>>,
}

impl CoreController {
    pub fn new(bus: Arc<dyn CanBus>) -> Self {
        Self {
            bus,
            devices: Arc::new(Mutex::new(HashMap::new())),
            polling_active: Arc::new(AtomicBool::new(false)),
            polling_thread: Mutex::new(None),
        }
    }

    pub fn bus(&self) -> Arc<dyn CanBus> {
        Arc::clone(&self.bus)
    }

    pub fn add_device(&self, device: Arc<dyn MotorDevice>) -> Result<()> {
        let motor_id = device.motor_id();
        {
            let mut devices = self
                .devices
                .lock()
                .map_err(|_| MotorError::Io("devices lock poisoned".to_string()))?;
            if devices.contains_key(&motor_id) {
                return Err(MotorError::InvalidArgument(format!(
                    "device with motor_id {motor_id} already exists"
                )));
            }
            devices.insert(motor_id, Arc::clone(&device));
        }

        self.start_polling_if_needed()?;
        Ok(())
    }

    pub fn poll_feedback_once(&self) -> Result<()> {
        while let Some(frame) = self.bus.recv(Duration::from_millis(0))? {
            if !frame.is_rx {
                continue;
            }
            let devices = self
                .devices
                .lock()
                .map_err(|_| MotorError::Io("devices lock poisoned".to_string()))?
                .values()
                .cloned()
                .collect::<Vec<_>>();
            for device in devices {
                if !device.accepts_frame(&frame) {
                    continue;
                }
                device.process_feedback_frame(frame)?;
                break;
            }
        }
        Ok(())
    }

    pub fn enable_all(&self) -> Result<()> {
        let devices = self
            .devices
            .lock()
            .map_err(|_| MotorError::Io("devices lock poisoned".to_string()))?
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for device in devices {
            device.enable()?;
        }
        Ok(())
    }

    pub fn disable_all(&self) -> Result<()> {
        let devices = self
            .devices
            .lock()
            .map_err(|_| MotorError::Io("devices lock poisoned".to_string()))?
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for device in devices {
            device.disable()?;
        }
        Ok(())
    }

    fn start_polling_if_needed(&self) -> Result<()> {
        if self.polling_active.load(Ordering::Acquire) {
            return Ok(());
        }

        self.polling_active.store(true, Ordering::Release);
        let active = Arc::clone(&self.polling_active);
        let bus = Arc::clone(&self.bus);
        let devices = Arc::clone(&self.devices);

        let handle = thread::spawn(move || {
            let idle_sleep = Duration::from_micros(200);
            while active.load(Ordering::Acquire) {
                match bus.recv(Duration::from_millis(0)) {
                    Ok(Some(frame)) => {
                        if frame.is_rx {
                            let snapshot = devices
                                .lock()
                                .ok()
                                .map(|m| m.values().cloned().collect::<Vec<_>>())
                                .unwrap_or_default();
                            for device in snapshot {
                                if !device.accepts_frame(&frame) {
                                    continue;
                                }
                                let _ = device.process_feedback_frame(frame);
                                break;
                            }
                        }
                        // Fast path: while frames are flowing, keep draining without sleeping.
                        continue;
                    }
                    Ok(None) => {
                        // Idle path: queue is empty, briefly yield CPU.
                        std::thread::sleep(idle_sleep);
                    }
                    Err(_) => active.store(false, Ordering::Release),
                }
            }
        });

        self.polling_thread
            .lock()
            .map_err(|_| MotorError::Io("polling thread lock poisoned".to_string()))?
            .replace(handle);

        Ok(())
    }

    pub fn shutdown(&self) -> Result<()> {
        self.close_inner(true)
    }

    pub fn close_bus(&self) -> Result<()> {
        self.close_inner(false)
    }

    fn close_inner(&self, disable_devices: bool) -> Result<()> {
        self.polling_active.store(false, Ordering::Release);
        if let Some(handle) = self
            .polling_thread
            .lock()
            .map_err(|_| MotorError::Io("polling thread lock poisoned".to_string()))?
            .take()
        {
            let _ = handle.join();
        }
        if disable_devices {
            let _ = self.disable_all();
        }
        self.bus.shutdown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::{CanBus, CanFrame};
    use crate::device::MotorDevice;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeBus {
        rx: Mutex<VecDeque<CanFrame>>,
        sent: Mutex<Vec<CanFrame>>,
        shutdown_count: AtomicUsize,
        fail_recv: AtomicBool,
    }

    impl FakeBus {
        fn new() -> Self {
            Self {
                rx: Mutex::new(VecDeque::new()),
                sent: Mutex::new(Vec::new()),
                shutdown_count: AtomicUsize::new(0),
                fail_recv: AtomicBool::new(false),
            }
        }

        fn push_rx(&self, frame: CanFrame) {
            self.rx.lock().expect("rx lock").push_back(frame);
        }

        fn set_fail_recv(&self, fail: bool) {
            self.fail_recv.store(fail, Ordering::SeqCst);
        }
    }

    impl CanBus for FakeBus {
        fn send(&self, frame: CanFrame) -> Result<()> {
            self.sent.lock().expect("sent lock").push(frame);
            Ok(())
        }

        fn recv(&self, _timeout: Duration) -> Result<Option<CanFrame>> {
            if self.fail_recv.load(Ordering::SeqCst) {
                return Err(MotorError::Io("injected recv error".to_string()));
            }
            Ok(self.rx.lock().expect("rx lock").pop_front())
        }

        fn shutdown(&self) -> Result<()> {
            self.shutdown_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct FakeDevice {
        id: u16,
        accepts_id: u32,
        enable_count: AtomicUsize,
        disable_count: AtomicUsize,
        processed_count: AtomicUsize,
    }

    impl FakeDevice {
        fn new(id: u16, accepts_id: u32) -> Self {
            Self {
                id,
                accepts_id,
                enable_count: AtomicUsize::new(0),
                disable_count: AtomicUsize::new(0),
                processed_count: AtomicUsize::new(0),
            }
        }
    }

    impl MotorDevice for FakeDevice {
        fn vendor(&self) -> &'static str {
            "fake"
        }

        fn model(&self) -> &str {
            "fake-model"
        }

        fn motor_id(&self) -> u16 {
            self.id
        }

        fn feedback_id(&self) -> u16 {
            self.accepts_id as u16
        }

        fn enable(&self) -> Result<()> {
            self.enable_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn disable(&self) -> Result<()> {
            self.disable_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn accepts_frame(&self, frame: &CanFrame) -> bool {
            frame.arbitration_id == self.accepts_id
        }

        fn process_feedback_frame(&self, _frame: CanFrame) -> Result<()> {
            self.processed_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn rx_frame(id: u32) -> CanFrame {
        CanFrame {
            arbitration_id: id,
            data: [0; 8],
            dlc: 8,
            is_extended: false,
            is_rx: true,
        }
    }

    #[test]
    fn add_device_rejects_duplicate_motor_id() {
        let bus = Arc::new(FakeBus::new());
        let ctrl = CoreController::new(bus);
        let d1: Arc<dyn MotorDevice> = Arc::new(FakeDevice::new(1, 0x11));
        let d2: Arc<dyn MotorDevice> = Arc::new(FakeDevice::new(1, 0x12));
        ctrl.add_device(d1).expect("first add");
        assert!(ctrl.add_device(d2).is_err());
        ctrl.close_bus().expect("close");
    }

    #[test]
    fn poll_feedback_routes_to_accepting_device() {
        let bus = Arc::new(FakeBus::new());
        bus.push_rx(rx_frame(0x12));
        bus.push_rx(rx_frame(0x11));

        let ctrl = CoreController::new(bus);
        let d1 = Arc::new(FakeDevice::new(1, 0x11));
        let d2 = Arc::new(FakeDevice::new(2, 0x12));
        ctrl.add_device(d1.clone()).expect("add d1");
        ctrl.add_device(d2.clone()).expect("add d2");

        ctrl.poll_feedback_once().expect("poll");

        assert_eq!(d1.processed_count.load(Ordering::SeqCst), 1);
        assert_eq!(d2.processed_count.load(Ordering::SeqCst), 1);
        ctrl.close_bus().expect("close");
    }

    #[test]
    fn enable_and_disable_all_touch_each_device_once() {
        let bus = Arc::new(FakeBus::new());
        let ctrl = CoreController::new(bus);
        let d1 = Arc::new(FakeDevice::new(1, 0x11));
        let d2 = Arc::new(FakeDevice::new(2, 0x12));
        ctrl.add_device(d1.clone()).expect("add d1");
        ctrl.add_device(d2.clone()).expect("add d2");

        ctrl.enable_all().expect("enable all");
        ctrl.disable_all().expect("disable all");

        assert_eq!(d1.enable_count.load(Ordering::SeqCst), 1);
        assert_eq!(d2.enable_count.load(Ordering::SeqCst), 1);
        assert_eq!(d1.disable_count.load(Ordering::SeqCst), 1);
        assert_eq!(d2.disable_count.load(Ordering::SeqCst), 1);
        ctrl.close_bus().expect("close");
    }

    #[test]
    fn shutdown_disables_devices_and_closes_bus() {
        let bus = Arc::new(FakeBus::new());
        let ctrl = CoreController::new(bus.clone());
        let d1 = Arc::new(FakeDevice::new(1, 0x11));
        ctrl.add_device(d1.clone()).expect("add d1");

        ctrl.shutdown().expect("shutdown");

        assert_eq!(d1.disable_count.load(Ordering::SeqCst), 1);
        assert_eq!(bus.shutdown_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn poll_feedback_once_returns_bus_recv_error() {
        let bus = Arc::new(FakeBus::new());
        let ctrl = CoreController::new(bus.clone());
        let d1: Arc<dyn MotorDevice> = Arc::new(FakeDevice::new(1, 0x11));
        ctrl.add_device(d1).expect("add d1");

        bus.set_fail_recv(true);
        let err = ctrl.poll_feedback_once().expect_err("recv should fail");
        assert!(err.to_string().contains("injected recv error"));
        ctrl.close_bus().expect("close");
    }
}
