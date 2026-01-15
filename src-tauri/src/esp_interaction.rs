use crate::models::ChipDetails;
use espflash::connection::{Connection, ResetAfterOperation, ResetBeforeOperation};
use espflash::flasher::Flasher;
use serialport::UsbPortInfo;

pub fn connect_and_get_info(port_name: &str) -> ChipDetails {
    // 1. Open Native Serial Port
    let serial_port = match serialport::new(port_name, 115200).open_native() {
        Ok(p) => p,
        Err(e) => {
            return ChipDetails {
                chip_model: None,
                mac_address: None,
                flash_size: None,
                features: None,
                crystal_frequency: None,
                chip_revision: None,
                error: Some(format!("Serial Error: {}", e)),
            }
        }
    };

    // 2. Find Port Info (Vital for Native USB support)
    // We must provide the correct VID/PID so espflash knows which reset strategy to use.
    let ports = serialport::available_ports().unwrap_or_default();
    let port_info = ports
        .iter()
        .find(|p| p.port_name == port_name)
        .map(|p| match &p.port_type {
            serialport::SerialPortType::UsbPort(info) => info.clone(),
            _ => UsbPortInfo {
                vid: 0,
                pid: 0,
                serial_number: None,
                manufacturer: None,
                product: None,
            },
        })
        .unwrap_or(UsbPortInfo {
            vid: 0,
            pid: 0,
            serial_number: None,
            manufacturer: None,
            product: None,
        });

    // 2. Create Connection
    let connection = Connection::new(
        serial_port,
        port_info,
        ResetAfterOperation::default(),
        ResetBeforeOperation::default(),
        115200,
    );

    // 3. Connect Flasher
    let mut flasher = match Flasher::connect(
        connection, true,  // load stub (Optimistically try true to fix connection error)
        false, // verify stub
        false, // force
        None,  // chip
        None,  // target_baud
    ) {
        Ok(f) => f,
        Err(e) => {
            return ChipDetails {
                chip_model: None,
                mac_address: None,
                flash_size: None,
                features: None,
                crystal_frequency: None,
                chip_revision: None,
                error: Some(format!("Connect Error: {}", e)),
            }
        }
    };

    // 4. Try to get info
    // Attempt to inspect flasher state
    let debug_info = format!("{:?}", flasher);

    // Use the chip trait to get the model dynamically
    let chip_model = Some(flasher.chip().to_string());

    let flash_size = if debug_info.contains("_16Mb") {
        Some("16 MB".to_string())
    } else if debug_info.contains("_8Mb") {
        Some("8 MB".to_string())
    } else if debug_info.contains("_4Mb") {
        Some("4 MB".to_string())
    } else {
        None
    };

    // Retrieve Device Info (MAC, Features, etc.)
    let (mac_address, features) = match flasher.device_info() {
        Ok(info) => {
            let mac = info.mac_address;

            // Probe for features
            let feats_probe = info.features;
            let feats_str = feats_probe
                .iter()
                .map(|f| format!("{:?}", f))
                .collect::<Vec<String>>()
                .join(", ");

            (mac, Some(feats_str))
        }
        Err(e) => {
            println!("Failed to get device info: {}", e);
            (None, None)
        }
    };

    // Get Chip Revision
    let chip_revision = match flasher.chip().revision(flasher.connection()) {
        Ok(rev) => Some(format!("v{}.{}", rev.0, rev.1)),
        Err(_) => None,
    };

    // Get Crystal Frequency
    let crystal_frequency = match flasher.chip().xtal_frequency(flasher.connection()) {
        Ok(freq) => Some(format!("{}", freq)),
        Err(_) => None,
    };

    println!("Debug Info: {}", debug_info);

    ChipDetails {
        chip_model,
        mac_address,
        flash_size,
        features,
        crystal_frequency,
        chip_revision,
        error: None,
    }
}

pub fn erase_flash(port_name: &str) -> Result<String, String> {
    // 1. Open Native Serial Port
    let serial_port = serialport::new(port_name, 115200)
        .open_native()
        .map_err(|e| format!("Serial Error: {}", e))?;

    // 2. Find Port Info
    let ports = serialport::available_ports().unwrap_or_default();
    let port_info = ports
        .iter()
        .find(|p| p.port_name == port_name)
        .map(|p| match &p.port_type {
            serialport::SerialPortType::UsbPort(info) => info.clone(),
            _ => UsbPortInfo {
                vid: 0,
                pid: 0,
                serial_number: None,
                manufacturer: None,
                product: None,
            },
        })
        .unwrap_or(UsbPortInfo {
            vid: 0,
            pid: 0,
            serial_number: None,
            manufacturer: None,
            product: None,
        });

    // 3. Create Connection
    let connection = Connection::new(
        serial_port,
        port_info,
        ResetAfterOperation::default(),
        ResetBeforeOperation::default(),
        115200,
    );

    // 4. Connect Flasher
    let mut flasher = Flasher::connect(
        connection, true,  // load stub
        false, // verify stub
        false, // force
        None,  // chip
        None,  // target_baud
    )
    .map_err(|e| format!("Connect Error: {}", e))?;

    // 5. Erase Flash
    println!("Erasing flash...");
    flasher
        .erase_flash()
        .map_err(|e| format!("Erase Error: {}", e))?;
    println!("Flash erased successfully");

    Ok("Flash Memory Erased Successfully".to_string())
}
