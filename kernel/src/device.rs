use anyhow::{anyhow, Result};
use hashbrown::HashMap;

use tinypci::{PciClass, PciDeviceInfo};

;

pub trait Driver {
    fn class(&self) -> PciClass;
}

struct Manager {
    drivers: HashMap<PciClass, &'static dyn Driver>,
}

impl Manager {
    fn add_driver(&mut self, driver: &'static dyn Driver) {
        self.drivers
            .insert(driver.class(), driver)
            .expect("driver for class already exists");
    }


    fn find_driver(&self, device: PciDeviceInfo) -> Result<&'static dyn Driver> {
        self.drivers
            .get(&device.class())
            .ok_or(anyhow!("couldn't find driver for device"))
            .copied()
    }
}
