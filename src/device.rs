use std::{
    cell::{RefCell, RefMut},
    error::Error,
    sync::Arc,
};

use append_only_vec::AppendOnlyVec;
use piet_common::Device;

#[derive(Clone)]
pub struct Pool {
    devices: Arc<AppendOnlyVec<RefCell<Device>>>,
}

impl Pool {
    pub fn new() -> Self {
        let v = AppendOnlyVec::new();
        Self {
            devices: Arc::new(v),
        }
    }

    pub fn get(&self) -> Result<RefMut<Device>, Box<dyn Error>> {
        if let Some(device) = self.devices.iter().find_map(|d| d.try_borrow_mut().ok()) {
            Ok(device)
        } else {
            let index = self.devices.push(RefCell::new(Device::new()?));
            let device = self.devices[index].borrow_mut();
            Ok(device)
        }
    }
}
