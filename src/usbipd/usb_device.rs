pub use std::str::FromStr;
use super::USBIPDError;

#[derive(Debug, Clone)]
pub struct UsbDevice {
    #[allow(dead_code)]
    pub instance_id: Option<String>,
    #[allow(dead_code)]
    pub hardware_id: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub is_forced: bool,
    #[allow(dead_code)]
    pub bus_id: Option<String>,
    #[allow(dead_code)]
    pub persisted_guid: Option<String>,
    #[allow(dead_code)]
    pub stub_instance_id: Option<String>,
    #[allow(dead_code)]
    pub client_ip_address: Option<String>,
    #[allow(dead_code)]
    pub is_bound: bool,
    #[allow(dead_code)]
    pub is_connected: bool,
    #[allow(dead_code)]
    pub is_attached: bool,
}

fn parse_bool(s: &str) -> Result<bool, String> {
    match s.to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Ok(false), // 默认返回 false
    }
}

// 辅助函数：处理可能为空的字符串
fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

impl FromStr for UsbDevice {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut instance_id = String::new();
        let mut hardware_id = String::new();
        let mut description = String::new();
        let mut is_forced = false;
        let mut bus_id = String::new();
        let mut persisted_guid = None;
        let mut stub_instance_id = None;
        let mut client_ip_address = None;
        let mut is_bound = false;
        let mut is_connected = false;
        let mut is_attached = false;

        for line in s.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "InstanceId" => instance_id = value.to_string(),
                    "HardwareId" => hardware_id = value.to_string(),
                    "Description" => description = value.to_string(),
                    "IsForced" => is_forced = parse_bool(value)?,
                    "BusId" => bus_id = value.to_string(),
                    "PersistedGuid" => persisted_guid = non_empty(value),
                    "StubInstanceId" => stub_instance_id = non_empty(value),
                    "ClientIPAddress" => client_ip_address = non_empty(value),
                    "IsBound" => is_bound = parse_bool(value)?,
                    "IsConnected" => is_connected = parse_bool(value)?,
                    "IsAttached" => is_attached = parse_bool(value)?,
                    _ => {}
                }
            }
        }

        Ok(UsbDevice {
            instance_id: non_empty(&instance_id),
            hardware_id: non_empty(&hardware_id),
            description: non_empty(&description),
            is_forced,
            bus_id: non_empty(&bus_id),
            persisted_guid,
            stub_instance_id,
            client_ip_address,
            is_bound,
            is_connected,
            is_attached,
        })
    }
}

// impl UsbDevice {
//     pub fn bind(&self) -> Result<bool, USBIPDError> {
//         if !self.is_bound {

//         } else {
//             Err("Device is already bound".to_string())
//         }
//     }
// }
