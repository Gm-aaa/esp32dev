use serde::Serialize;

#[derive(Serialize)]
pub struct DeviceStatus {
    pub code: String, // "ok", "missing_driver", "none"
    pub message: String,
    pub port_name: Option<String>,
    pub product_name: Option<String>,
    pub serial_number: Option<String>,
    pub vid_pid: Option<String>,
    pub connection_type: Option<String>,
}

#[derive(Serialize)]
pub struct ChipDetails {
    pub chip_model: Option<String>,
    pub mac_address: Option<String>,
    pub flash_size: Option<String>,
    pub features: Option<String>,
    pub chip_revision: Option<String>,
    pub error: Option<String>,
}
