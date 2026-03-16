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
    devices_by_feedback_logical_id: Arc<Mutex<HashMap<u8, Arc<dyn MotorDevice>>>>,
    polling_active: Arc<AtomicBool>,
    polling_thread: Mutex<Option<JoinHandle<()>>>,
}

impl CoreController {
    pub fn new(bus: Arc<dyn CanBus>) -> Self {
        Self {
            bus,
            devices: Arc::new(Mutex::new(HashMap::new())),
            devices_by_feedback_logical_id: Arc::new(Mutex::new(HashMap::new())),
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

        self
            .devices_by_feedback_logical_id
            .lock()
            .map_err(|_| MotorError::Io("feedback map lock poisoned".to_string()))?
            .insert(device.feedback_logical_id(), device);

        self.start_polling_if_needed()?;
        Ok(())
    }

    pub fn poll_feedback_once(&self) -> Result<()> {
        while let Some(frame) = self.bus.recv(Duration::from_millis(0))? {
            if !frame.is_rx {
                continue;
            }
            let logical_id = frame.data[0] & 0x0F;
            if let Some(device) = self
                .devices_by_feedback_logical_id
                .lock()
                .map_err(|_| MotorError::Io("feedback map lock poisoned".to_string()))?
                .get(&logical_id)
                .cloned()
            {
                device.process_feedback_frame(frame)?;
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
        let feedback_map = Arc::clone(&self.devices_by_feedback_logical_id);

        let handle = thread::spawn(move || {
            while active.load(Ordering::Acquire) {
                match bus.recv(Duration::from_millis(1)) {
                    Ok(Some(frame)) => {
                        if frame.is_rx {
                            let logical_id = frame.data[0] & 0x0F;
                            if let Some(device) = feedback_map
                                .lock()
                                .ok()
                                .and_then(|m| m.get(&logical_id).cloned())
                            {
                                let _ = device.process_feedback_frame(frame);
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(_) => active.store(false, Ordering::Release),
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        });

        self
            .polling_thread
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
