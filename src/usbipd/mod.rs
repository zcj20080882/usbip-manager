
pub mod usb_device;
pub mod error;
pub mod runner;
pub mod commands;

use winreg::{enums::*, RegKey};
use std::collections::HashMap;
use tokio::sync::Mutex as AsyncMutex;
use std::sync::Arc;
use once_cell::sync::Lazy;

#[allow(unused_imports)]
pub use tracing::{info, warn, error, debug, trace};
#[allow(unused_imports)]
pub use self::usb_device::UsbDevice;
pub use self::error::Error as USBIPDError;

static USBIPD_INSTALLATION_PATH: Lazy<Arc<AsyncMutex<String>>> = Lazy::new(|| Arc::new(AsyncMutex::new(String::new())));
static USBIPD_VERSION: Lazy<Arc<AsyncMutex<String>>> = Lazy::new(|| Arc::new(AsyncMutex::new(String::new())));
static USBIPD_GET_DEVICES_SCRIPT: Lazy<Arc<AsyncMutex<String>>> = Lazy::new(|| Arc::new(AsyncMutex::new(String::new())));


const WIN64_UNINSTALL_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
const WIN32_UNINSTALL_KEY: &str = r"SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall";
const USBIPD4_GET_DEVICES_SCRIPT: &str = r"Import-Module $env:ProgramW6432'\usbipd-win\PowerShell\Usbipd.Powershell.dll';Get-UsbipdDevice";
const USBIPD5_GET_DEVICES_SCRIPT: &str = r"Import-Module $env:ProgramW6432'\usbipd-win\Usbipd.Powershell.dll';Get-UsbipdDevice";


fn check_registry_key(key: &RegKey, app_name: &str) -> Result<Option<HashMap<String, String>>, USBIPDError> {
    for subkey_name in key.enum_keys() {

        if let Ok(subkey_name) = subkey_name {
            if let Ok(subkey) = key.open_subkey(&subkey_name) {
                if let Ok(display_name) = subkey.get_value::<String, _>("DisplayName") {
                    if display_name.contains(app_name) {
                        let mut info = HashMap::new();

                        if let Ok(install_location) = subkey.get_value::<String, _>("InstallLocation") {
                            info.insert("InstallLocation".to_string(), install_location);
                        }

                        if let Ok(display_version) = subkey.get_value::<String, _>("DisplayVersion") {
                            info.insert("DisplayVersion".to_string(), display_version);
                        }

                        if let Ok(publisher) = subkey.get_value::<String, _>("Publisher") {
                            info.insert("Publisher".to_string(), publisher);
                        }

                        return Ok(Some(info));
                    }
                }
            }
        }
    }

    Err(USBIPDError::UsbIPDNotFound)
}


async fn usbipd_installation_check() -> Result<bool, USBIPDError> {

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let app_name = "usbipd-win";
    let mut install_location:String = String::new();
    let mut display_version:String = String::new();

    if !USBIPD_INSTALLATION_PATH.lock().await.is_empty() && !USBIPD_VERSION.lock().await.is_empty() {
        return Ok(true);
    }

    // Check 64-bit applications
    if let Ok(key) = hklm.open_subkey(WIN64_UNINSTALL_KEY){
        if let Ok(Some(result)) = check_registry_key(&key, app_name) {
            info!("Found usbipd in 64-bit uninstall key.\n");
            info!("InstallLocation: {}\n", result.get("InstallLocation").unwrap_or(&"N/A".to_string()));
            info!("DisplayVersion: {}\n", result.get("DisplayVersion").unwrap_or(&"N/A".to_string()));
            install_location.push_str(result.get("InstallLocation").unwrap_or(&"".to_string()));
            display_version.push_str(result.get("DisplayVersion").unwrap_or(&"".to_string()));
        }
    }

    if install_location.is_empty() || display_version.is_empty() {
        warn!("usbipd not found in 64-bit uninstall key, checking 32-bit...\n");
        if let Ok(key) = hklm.open_subkey(WIN32_UNINSTALL_KEY) {
            if let Ok(Some(result)) = check_registry_key(&key, app_name) {
                install_location.clear();
                display_version.clear();
                install_location.push_str(result.get("InstallLocation").unwrap_or(&"".to_string()));
                display_version.push_str(result.get("DisplayVersion").unwrap_or(&"".to_string()));
                info!("Found usbipd in 64-bit uninstall key.\n");
            }
        } else {
            error!("Failed to open uninstall key.\n");
            return Err(USBIPDError::UsbIPDNotFound);
        }
    }

    if install_location.is_empty() || display_version.is_empty() {
        error!("{} is not installed or missing information.\n", app_name);
        return Err(USBIPDError::UsbIPDNotFound);
    }

    let parts: Vec<&str> = display_version.split('.').collect();
    if parts.len() < 2 {
        error!("usbipd version is too low: {}\n", display_version);
        return Err(USBIPDError::UsbIPDLowVersion);
    }
    let major = parts[0].parse::<u32>().unwrap_or(0);
    if major < 4  {
        error!("usbipd version is too low: {}\n", display_version);
        return Err(USBIPDError::UsbIPDLowVersion);
    }
    else if major >= 4 && major < 5 {
        info!("usbipd version is 4.x, using usbipd4 PowerShell script.\n");
        USBIPD_GET_DEVICES_SCRIPT.lock().await.push_str(USBIPD4_GET_DEVICES_SCRIPT);
    }
    else {
        info!("usbipd version is 5.x, using usbipd5 PowerShell script.\n");
        USBIPD_GET_DEVICES_SCRIPT.lock().await.push_str(USBIPD5_GET_DEVICES_SCRIPT);
    }

    USBIPD_INSTALLATION_PATH.lock().await.clear();
    USBIPD_INSTALLATION_PATH.lock().await.push_str(install_location.as_str());
    USBIPD_VERSION.lock().await.clear();
    USBIPD_VERSION.lock().await.push_str(display_version.as_str());

    debug!("usbipd installation path: {}\n", USBIPD_INSTALLATION_PATH.lock().await);
    debug!("usbipd version: {}\n", USBIPD_VERSION.lock().await);
    Ok(true)
}

pub async fn get_usbipd_powershell_script() -> Result<String, USBIPDError> {
    if USBIPD_GET_DEVICES_SCRIPT.lock().await.is_empty() {
        usbipd_installation_check().await?;
        if USBIPD_GET_DEVICES_SCRIPT.lock().await.is_empty() {
            error!("USBIPD PowerShell script is empty, please check usbipd installation.\n");
            return Err(USBIPDError::UsbIPDNotFound);
        }
    }
    Ok(USBIPD_GET_DEVICES_SCRIPT.lock().await.clone())
}
