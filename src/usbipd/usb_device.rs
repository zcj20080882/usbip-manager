#[allow(unused_imports)]
use super::USBIPD;
#[allow(unused_imports)]
use super::USBIPDError::{Error, ErrorCode, USBIPDResult};
use regex::Regex;
use serde::{Deserialize};

use std::thread::sleep;
use std::time::Duration;
use std::sync::{Arc, Mutex};
#[allow(unused_imports)]
pub use tracing::{debug, error, info, trace, warn};
#[derive(Debug, Clone, Deserialize)]
pub struct UsbDevice {
    #[serde(skip)]
    pub usbipd: Arc<Mutex<USBIPD>>,
    #[serde(skip)]
    pub is_connected: bool,
    #[serde(skip)]
    pub is_bound: bool,
    #[serde(skip)]
    pub is_attached: bool,
    #[serde(skip)]
    pub hardware_id: Option<String>,
    #[serde(rename = "BusId")]
    bus_id: Option<String>,

    #[serde(rename = "ClientIPAddress")]
    client_ip_address: Option<String>,

    #[serde(rename = "Description")]
    description: Option<String>,

    #[serde(rename = "InstanceId")]
    instance_id: Option<String>,

    #[serde(rename = "IsForced")]
    is_forced: bool,

    #[serde(rename = "PersistedGuid")]
    persisted_guid: Option<String>,

    #[serde(rename = "StubInstanceId")]
    stub_instance_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UsbDevices {
    #[serde(rename = "Devices")]
    pub devices: Vec<UsbDevice>,
}

pub trait FromJson: Sized {
    fn from_json(s: &str, usbipd: Arc<Mutex<USBIPD>>) -> Result<Self, Error>;
}

impl UsbDevice {
    #[allow(dead_code)]
    pub fn bind(&self) -> Result<bool, Error> {
        if self.is_bound {
            warn!(
                "Device {} is already bound, no need to bind again.",
                self.hardware_id.as_deref().unwrap_or("Unknown")
            );
            return Ok(true);
        }

        let hardware_id = match &self.hardware_id {
            Some(id) => id,
            None => {
                error!(
                    "Device {} has no hardware ID, cannot bind.",
                    self.hardware_id.as_deref().unwrap_or("Unknown")
                );
                return Err(Error::new(
                    ErrorCode::InternalError,
                    "Device has no hardware ID".to_string(),
                ));
            }
        };
        let usbipd = self.usbipd.lock().unwrap();
        let result = usbipd.bind_device(hardware_id, false)?;
        info!("Device bind result: {}", result);
        sleep(Duration::from_millis(100));
        Ok(true)
    }

    #[allow(dead_code)]
    pub fn unbind(&self) -> Result<bool, Error> {
        if !self.is_bound {
            return Ok(true);
        }

        let hardware_id = match &self.hardware_id {
            Some(id) => id,
            None => {
                error!(
                    "Device {} has no hardware ID, cannot unbind.",
                    self.hardware_id.as_deref().unwrap_or("Unknown")
                );
                return Err(Error::new(
                    ErrorCode::InternalError,
                    "Device has no hardware ID".to_string(),
                ));
            }
        };
        let usbipd = self.usbipd.lock().unwrap();
        let result = usbipd.unbind_device(Some(hardware_id))?;
        info!("Device unbind result: {}", result);
        sleep(Duration::from_millis(100));
        Ok(true)
    }

    #[allow(dead_code)]
    pub fn attach(&self, host_ip: Option<&str>) -> Result<bool, Error> {
        if !self.is_bound {
            return Err(Error::new(
                ErrorCode::NotBound,
                "Device is not bound".to_string(),
            ));
        }
        if self.is_attached {
            return Ok(true);
        }

        let hardware_id = match &self.hardware_id {
            Some(id) => id,
            None => {
                error!(
                    "Device {} has no bus ID, cannot attach.",
                    self.hardware_id.as_deref().unwrap_or("Unknown")
                );
                return Err(Error::new(
                    ErrorCode::InternalError,
                    "Device has no bus ID".to_string(),
                ));
            }
        };
        let usbipd = self.usbipd.lock().unwrap();
        let result = usbipd.attach_device(hardware_id, host_ip)?;
        info!("Device attach result: {}", result);
        sleep(Duration::from_millis(100));
        Ok(true)
    }

    #[allow(dead_code)]
    pub fn detach(&self) -> Result<bool, Error> {
        if !self.is_attached {
            return Ok(true);
        }

        let hardware_id = match &self.hardware_id {
            Some(id) => id,
            None => {
                error!(
                    "Device {} has no bus ID, cannot detach.",
                    self.hardware_id.as_deref().unwrap_or("Unknown")
                );
                return Err(Error::new(
                    ErrorCode::InternalError,
                    "Device has no bus ID".to_string(),
                ));
            }
        };
        let usbipd = self.usbipd.lock().unwrap();
        let result = usbipd.detach_device(hardware_id)?;
        info!("Device detach result: {}", result);
        sleep(Duration::from_millis(100));
        Ok(true)
    }

    #[allow(dead_code)]
    pub fn update(&mut self) -> Result<bool, Error> {
        let usbipd = self.usbipd.lock().unwrap();
        let dev_list = usbipd.get_all_devices()?;
        for dev in dev_list.iter() {
            if dev.bus_id == self.bus_id {
                self.is_connected = dev.is_connected;
                self.is_bound = dev.is_bound;
                self.is_attached = dev.is_attached;
                self.hardware_id = dev.hardware_id.clone();
                self.bus_id = dev.bus_id.clone();
                self.client_ip_address = dev.client_ip_address.clone();
                self.description = dev.description.clone();
                self.instance_id = dev.instance_id.clone();
                self.is_forced = dev.is_forced;
                self.persisted_guid = dev.persisted_guid.clone();
                self.stub_instance_id = dev.stub_instance_id.clone();
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn get_hardware_id_from_instance_id(instance_id: Option<String>) -> Option<String> {
    if let Some(id) = &instance_id {
        let re = Regex::new(r"VID_([0-9A-Fa-f]+)&PID_([0-9A-Fa-f]+)").unwrap();
        if let Some(caps) = re.captures(id) {
            let vid = u16::from_str_radix(&caps[1], 16).ok()?;
            let pid = u16::from_str_radix(&caps[2], 16).ok()?;

            return Some(format!("{:04X}:{:04X}", vid, pid));
        }
    }
    None
}

impl FromJson for UsbDevices {
    fn from_json(s: &str, usbipd: Arc<Mutex<USBIPD>>) -> Result<Self, Error> {
        match serde_json::from_str::<UsbDevices>(s) {
            Ok(mut devices) => {
                // If you want to set hardware_id based on instance_id, you need to make hardware_id mutable and public.
                for device in &mut devices.devices {
                    device.usbipd = Arc::clone(&usbipd);
                    device.hardware_id =
                        get_hardware_id_from_instance_id(device.instance_id.clone());
                    device.is_connected = device.bus_id.is_some();
                    device.is_bound = device.is_connected && device.persisted_guid.is_some();
                    device.is_attached = device.is_connected
                        && device.is_bound
                        && device.client_ip_address.is_some();
                }
                Ok(devices)
            }
            Err(e) => Err(Error::new(ErrorCode::ParseError, e.to_string())),
        }
    }
}
