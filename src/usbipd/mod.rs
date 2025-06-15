pub mod commands;
pub mod error;
pub mod runner;
pub mod usb_device;
pub mod wsl;

use std::collections::HashMap;
use std::path::Path;
use winreg::{RegKey, enums::*};

#[allow(unused_imports)]
pub use tracing::{debug, error, info, trace, warn};

pub use self::error as USBIPDError;
#[allow(unused_imports)]
pub use self::usb_device::UsbDevice;
#[allow(unused_imports)]
use USBIPDError::{Error, ErrorCode, USBIPDResult};

const WIN64_UNINSTALL_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
const WIN32_UNINSTALL_KEY: &str =
    r"SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall";

#[derive(Debug, Default, Clone)]
pub struct USBIPD {
    #[allow(dead_code)]
    path: String,
    #[allow(dead_code)]
    version: String,
    pub count: i32,
}
fn check_registry_key(
    key: &RegKey,
    app_name: &str,
) -> Result<Option<HashMap<String, String>>, ErrorCode> {
    for subkey_name in key.enum_keys() {
        if let Ok(subkey_name) = subkey_name {
            if let Ok(subkey) = key.open_subkey(&subkey_name) {
                if let Ok(display_name) = subkey.get_value::<String, _>("DisplayName") {
                    if display_name.contains(app_name) {
                        let mut info = HashMap::new();

                        if let Ok(install_location) =
                            subkey.get_value::<String, _>("InstallLocation")
                        {
                            info.insert("InstallLocation".to_string(), install_location);
                        }

                        if let Ok(display_version) = subkey.get_value::<String, _>("DisplayVersion")
                        {
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

    Err(ErrorCode::UsbIPDNotFound)
}

impl USBIPD {
    pub fn new() -> Result<Self, Error> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let app_name = "usbipd-win";

        // Helper to extract install info from registry
        fn get_install_info(
            hklm: &RegKey,
            key_path: &str,
            app_name: &str,
        ) -> Option<HashMap<String, String>> {
            hklm.open_subkey(key_path)
                .ok()
                .and_then(|key| check_registry_key(&key, app_name).ok().flatten())
        }

        // Try 64-bit first, then 32-bit
        let install_info = get_install_info(&hklm, WIN64_UNINSTALL_KEY, app_name)
            .or_else(|| {
                warn!("usbipd not found in 64-bit uninstall key, checking 32-bit...");
                get_install_info(&hklm, WIN32_UNINSTALL_KEY, app_name)
            })
            .ok_or_else(|| {
                error!("{} is not installed or missing information.", app_name);
                Error {
                    code: ErrorCode::UsbIPDNotFound,
                    message: format!("{} is not installed or missing information.", app_name),
                }
            })?;

        let install_location = install_info
            .get("InstallLocation")
            .cloned()
            .unwrap_or_default();
        let display_version = install_info
            .get("DisplayVersion")
            .cloned()
            .unwrap_or_default();

        if install_location.is_empty() || display_version.is_empty() {
            error!("{} is not installed or missing information.", app_name);
            return Err(Error {
                code: ErrorCode::UsbIPDNotFound,
                message: format!("{} is not installed or missing information.", app_name),
            });
        }

        let usbipd_path = Path::new(&install_location).join("usbipd.exe");
        let usbipd_path_str = usbipd_path.to_str().ok_or_else(|| {
            error!("usbipd installation path is invalid: {:?}", usbipd_path);
            Error {
                code: ErrorCode::UsbIPDNotFound,
                message: format!("usbipd installation path is invalid: {:?}", usbipd_path),
            }
        })?;

        if !usbipd_path.is_file() {
            error!(
                "usbipd installation path does not exist or is not a file: {:?}",
                usbipd_path_str
            );
            return Err(Error {
                code: ErrorCode::UsbIPDNotFound,
                message: format!(
                    "usbipd installation path does not exist or is not a file: {:?}",
                    usbipd_path_str
                ),
            });
        }

        // Version check
        let mut parts = display_version
            .split('.')
            .filter_map(|s| s.parse::<u32>().ok());
        let major = parts.next().unwrap_or(0);
        if major < 5 {
            error!("usbipd version is too low: {}", display_version);
            return Err(Error {
                code: ErrorCode::UsbIPDLowVersion,
                message: format!("usbipd version is too low: {}", display_version),
            });
        }

        // debug!("usbipd installation path: {}", usbipd_path_str);
        // debug!("usbipd version: {}", display_version);

        Ok(USBIPD {
            path: usbipd_path_str.to_string(),
            version: display_version,
            count: 0,
        })
    }

    #[allow(dead_code)]
    pub fn path(&self) -> Result<&str, ErrorCode> {
        if self.path.is_empty() {
            error!("usbipd installation path is empty.\n");
            return Err(ErrorCode::UsbIPDNotFound);
        }
        Ok(&self.path)
    }

    #[allow(dead_code)]
    pub fn version(&self) -> Result<&str, ErrorCode> {
        if self.version.is_empty() {
            error!("usbipd version is empty.\n");
            return Err(ErrorCode::UsbIPDNotFound);
        }
        Ok(&self.version)
    }
}
