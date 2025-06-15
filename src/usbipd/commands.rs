use crate::usbipd::usb_device::FromJson;

use super::USBIPD;
#[allow(unused_imports)]
use super::USBIPDError::{Error, ErrorCode, USBIPDResult};
use super::usb_device::{UsbDevice, UsbDevices};
use std::sync::{Arc, Mutex};
use tracing::{debug, error};

impl USBIPD {
    #[allow(dead_code)]
    pub fn get_all_devices(&self) -> Result<Vec<UsbDevice>, Error> {
        let output = self.run_usbipd_command(vec!["state"], false).map_err(|e| {
            error!("Failed to list USB devices: {}", e);
            e
        })?;

        UsbDevices::from_json(&output, Arc::new(Mutex::new(self.clone())))
            .map(|devices| {
                debug!("Parsed USB devices successfully");
                devices.devices
            })
            .map_err(|e| {
                error!("Failed to parse {}: {}",output, e);
                Error::new(ErrorCode::ParseError, e.to_string())
            })
    }

    #[allow(dead_code)]
    pub fn bind_device(&self, hardware_id: &str, forced: bool) -> Result<bool, Error> {
        let args = if forced {
            vec!["bind", "--hardware-id", hardware_id, "--force"]
        } else {
            vec!["bind", "--hardware-id", hardware_id]
        };
        debug!("Binding USB device with hardware ID: {}\n", hardware_id);
        return match self.run_usbipd_command(args, true) {
            Ok(_) => {
                Ok(true)
            }
            Err(e) => {
                error!("Failed to bind USB device: {}", e);
                Err(e)
            }
        };
    }

    #[allow(dead_code)]
    pub fn unbind_device(&self, hardware_id: Option<&str>) -> Result<bool, Error> {
        let args = if let Some(id) = hardware_id {
            debug!("Unbinding USB device with hardware ID: {}\n", id);
            vec!["unbind", "--hardware-id", id]
        } else {
            vec!["unbind", "--all"]
        };

        return match self.run_usbipd_command(args, true) {
            Ok(_) => {
                Ok(true)
            }
            Err(e) => {
                error!("Failed to unbind USB device: {}", e);
                Err(e)
            }
        };
    }

    #[allow(dead_code)]
    pub fn attach_device(&self, hardware_id: &str, host_ip: Option<&str>) -> Result<bool, Error> {
        let distribution = self.get_default_or_first_running_wsl_distribution()?;
        let args = if let Some(ip) = host_ip {
            debug!("Attaching USB device with host ip: {}\n", ip);
            vec![
                "attach",
                "--hardware-id",
                hardware_id,
                "--host-ip",
                ip,
                "--wsl",
                distribution.as_str(),
            ]
        } else {
            vec!["attach", "--hardware-id", hardware_id, "--wsl", distribution.as_str()]
        };

        return match self.run_usbipd_command(args, false) {
            Ok(_) => {
                Ok(true)
            }
            Err(e) => {
                error!("Failed to unbind USB device: {}", e);
                Err(e)
            }
        };
    }

    #[allow(dead_code)]
    pub fn detach_device(&self, hardware_id: &str) -> Result<bool, Error> {
        let args = vec!["detach", "--hardware-id", hardware_id];

        return match self.run_usbipd_command(args, false) {
            Ok(_) => {
                Ok(true)
            }
            Err(e) => {
                error!("Failed to unbind USB device: {}", e);
                Err(e)
            }
        };
    }
}
