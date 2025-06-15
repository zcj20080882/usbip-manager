
use super::USBIPDError;
use super::usb_device::UsbDevice;
use super::runner as runner;
use tracing::{debug, error};
use std::str::FromStr;

pub async fn get_all_devices() -> Result<Vec<UsbDevice>, USBIPDError>
{
    let script = super::get_usbipd_powershell_script()
        .await
        .map_err(|_| USBIPDError::UsbIPDNotFound)?;
    let mut devices = Vec::new();
    debug!("Running PowerShell script to get all USB devices...\n");
    match runner::run_powershell_script(script.as_str()).await{
        Ok(output) => {
            debug!("PowerShell script output:\n{}", output);

            let mut current_block = String::new();

            // Process input line by line
            for line in output.lines() {
                let line = line.trim();

                if line.is_empty() {
                    // Encountered an empty line: if the current block is not empty, parse and reset
                    if !current_block.is_empty() {
                        let device = UsbDevice::from_str(&current_block)
                            .expect("Failed to parse USB device from block");
                        devices.push(device);
                        current_block.clear();
                    }
                } else {
                    // Non-empty line: add to the current block, preserving the original newline
                    current_block.push_str(line);
                    current_block.push('\n');
                }
            }

            // Process the last block
            if !current_block.is_empty() {
                let device = UsbDevice::from_str(&current_block)
                    .expect("Failed to parse USB device from block");
                devices.push(device);
            }
            debug!("Found {} USB devices:\n", devices.len());
        },
        Err(e) => {
            error!("Error running PowerShell script: {}", e);
            return Err(e);
        }
    };
    Ok(devices)
}

pub async fn bind_device(hardware_id: &str, forced: bool) -> Result<bool, USBIPDError> {
    let args = if forced {
        vec!["bind","--hardware-id", hardware_id,"--force"]
    } else {
        vec!["bind","--hardware-id", hardware_id]
    };
    debug!("Binding USB device with hardware ID: {}\n", hardware_id);
    return match runner::run_usbipd_command(args, true).await {
        Ok(output) => {
            debug!("USB device bound successfully: {}", output);
            Ok(true)
        },
        Err(e) => {
            error!("Failed to bind USB device: {}", e);
            Err(e)
        }
    };
}

pub async fn unbind_device(hardware_id: Option<&str>) -> Result<bool, USBIPDError> {
    let args = if let Some(id) = hardware_id {
        debug!("Unbinding USB device with hardware ID: {}\n", id);
        vec!["unbind","--hardware-id", id]
    } else {
        vec!["unbind","--all"]
    };

    return match runner::run_usbipd_command(args, true).await {
        Ok(output) => {
            debug!("USB device unbound successfully: {}", output);
            Ok(true)
        },
        Err(e) => {
            error!("Failed to unbind USB device: {}", e);
            Err(e)
        }
    };
}
// pub async fn attach_device(bus_id: &str, auto: bool, host_ip: &str) -> Result<String, USBIPDError> {
//     let script = super::get_usbipd_powershell_script()
//         .await
//         .map_err(|_| USBIPDError::UsbIPDNotFound)?;
//     debug!("Attaching USB device with instance ID: {}\n", instance_id);
//     let command = format!("{} -InstanceId '{}'", script, instance_id);
//     runner::run_usbipd_command(["attach","--har"].to_vec(),false).await
// }
