use std::{
    error::Error,
    sync::{Arc, Mutex, MutexGuard},
};

use append_only_vec::AppendOnlyVec;
use piet_common::Device;

#[derive(Clone)]
pub struct Pool {
    devices: Arc<AppendOnlyVec<Mutex<Device>>>,
}

impl Default for Pool {
    fn default() -> Self {
        Self::new()
    }
}

impl Pool {
    pub fn new() -> Self {
        let v = AppendOnlyVec::new();
        Self {
            devices: Arc::new(v),
        }
    }

    pub fn get(&self) -> Result<MutexGuard<Device>, Box<dyn Error>> {
        if let Some(device) = self.devices.iter().find_map(|d| d.try_lock().ok()) {
            Ok(device)
        } else {
            let index = self.devices.push(Mutex::new(Device::new()?));
            match self.devices[index].try_lock() {
                Ok(device) => Ok(device),
                Err(std::sync::TryLockError::WouldBlock) => self.get(),
                Err(e) => Err(e).unwrap(),
            }
        }
    }
}
