mod esp_interaction;
mod models;

use models::{ChipDetails, DeviceStatus};
use serialport::SerialPortType;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn check_device_status() -> DeviceStatus {
    // 1. Try to find ESP32 in COM ports
    if let Ok(ports) = serialport::available_ports() {
        for p in ports {
            if let SerialPortType::UsbPort(info) = p.port_type {
                // Check for common ESP32 USB to UART bridge Vendor IDs
                if [0x10C4, 0x1A86, 0x303A, 0x0403].contains(&info.vid) {
                    return DeviceStatus {
                        code: "ok".to_string(),
                        message: format!("Connected ({})", p.port_name),
                        port_name: Some(p.port_name),
                        product_name: info.product,
                        serial_number: info.serial_number,
                        vid_pid: Some(format!("{:04X}:{:04X}", info.vid, info.pid)),
                        connection_type: Some(if info.vid == 0x303A {
                            "native_usb".to_string()
                        } else {
                            "uart_bridge".to_string()
                        }),
                    };
                }
            }
        }
    }

    // 2. If no COM port found, check USB bus for missing drivers
    if let Ok(devices) = nusb::list_devices() {
        for dev in devices {
            let vid = dev.vendor_id();
            let pid = dev.product_id();
            if [0x10C4, 0x1A86, 0x303A, 0x0403].contains(&vid) {
                return DeviceStatus {
                    code: "missing_driver".to_string(),
                    message: "Driver Missing".to_string(),
                    port_name: None,
                    product_name: dev.product_string().map(|s| s.to_string()),
                    serial_number: dev.serial_number().map(|s| s.to_string()),
                    vid_pid: Some(format!("{:04X}:{:04X}", vid, pid)),
                    connection_type: Some(if vid == 0x303A {
                        "native_usb".to_string()
                    } else {
                        "uart_bridge".to_string()
                    }),
                };
            }
        }
    }

    // 3. No device found
    DeviceStatus {
        code: "none".to_string(),
        message: "Disconnected".to_string(),
        port_name: None,
        product_name: None,
        serial_number: None,
        vid_pid: None,
        connection_type: None,
    }
}

#[tauri::command]
async fn get_chip_info(port_name: String) -> ChipDetails {
    esp_interaction::connect_and_get_info(&port_name)
}

#[tauri::command]
async fn check_ch34x_driver() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let output = std::process::Command::new("driverquery")
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_lowercase();
                stdout.contains("ch34")
            }
            Err(_) => false,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // On non-Windows, assume true or handle differently.
        // For now, returning true to avoid showing "Driver Missing" on Mac/Linux where this check doesn't apply.
        true
    }
}

#[tauri::command]
async fn flash_firmware(
    port_name: String,
    firmware_path: String,
    flash_address: String,
) -> Result<String, String> {
    // Placeholder for actual flashing logic
    // This requires spawning a separate task and managing state
    println!(
        "Flashing request: {} -> {} @ {}",
        firmware_path, port_name, flash_address
    );
    // Simulate delay
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok("Flash started (Stub)".to_string())
}

#[tauri::command]
async fn erase_flash(port_name: String) -> Result<String, String> {
    // Run in a blocking task because it blocks the thread
    tauri::async_runtime::spawn_blocking(move || esp_interaction::erase_flash(&port_name))
        .await
        .map_err(|e| e.to_string())?
}

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{Emitter, State};

pub struct SerialState {
    port: Arc<Mutex<Option<Box<dyn serialport::SerialPort>>>>,
    should_run: Arc<Mutex<bool>>,
}

#[tauri::command]
async fn monitor_connect(
    app: tauri::AppHandle,
    state: State<'_, SerialState>,
    port_name: String,
    baud_rate: u32,
) -> Result<String, String> {
    let mut serial_port = serialport::new(&port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .map_err(|e| format!("Failed to open port: {}", e))?;

    // ESP32 requires DTR=false, RTS=false to run normally
    serial_port.write_data_terminal_ready(false).ok();
    serial_port.write_request_to_send(false).ok();

    // Set run flag
    {
        let mut run = state.should_run.lock().unwrap();
        *run = true;
    }

    // Store port (wrap in Arc/Mutex logic)
    {
        let mut port_guard = state.port.lock().unwrap();
        *port_guard = Some(serial_port);
    }

    // Clone Arcs for thread (cheap clone)
    let port_clone = state.port.clone();
    let run_clone = state.should_run.clone();
    let port_name_thread = port_name.clone();
    let baud_rate_thread = baud_rate;

    // Spawn read thread
    std::thread::spawn(move || {
        let mut serial_buf: Vec<u8> = vec![0; 1000];
        loop {
            // Check run flag
            if !*run_clone.lock().unwrap() {
                break;
            }

            let mut fatal_error = false;
            let mut got_data = false;
            let mut read_len = 0;

            // Scope for lock
            {
                let mut guard = port_clone.lock().unwrap();
                if let Some(port) = guard.as_mut() {
                    match port.read(serial_buf.as_mut_slice()) {
                        Ok(t) => {
                            if t > 0 {
                                got_data = true;
                                read_len = t;
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                        Err(e) => {
                            println!("Monitor Error: {:?} - triggering reconnect", e);
                            fatal_error = true;
                        }
                    }
                } else {
                    // Port is None, need reconnect
                    fatal_error = true;
                }

                if fatal_error {
                    *guard = None;
                }
            }

            if got_data {
                println!("Serial Read {} bytes", read_len);
                let data = String::from_utf8_lossy(&serial_buf[..read_len]).to_string();
                let _ = app.emit("serial-read", data);
            }

            if fatal_error {
                // Wait before retrying
                std::thread::sleep(Duration::from_millis(500));

                println!("Attempting reconnect to {}...", port_name_thread);
                match serialport::new(&port_name_thread, baud_rate_thread)
                    .timeout(Duration::from_millis(10))
                    .open()
                {
                    Ok(mut new_port) => {
                        new_port.write_data_terminal_ready(false).ok();
                        new_port.write_request_to_send(false).ok();

                        let mut guard = port_clone.lock().unwrap();
                        *guard = Some(new_port);
                        println!("Reconnected successfully!");
                    }
                    Err(_) => {
                        // Reconnect failed, just retry next loop
                    }
                }
            } else {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        println!("Monitor thread stopped");
    });

    println!("Monitor connect: {} @ {}", port_name, baud_rate);
    Ok("Connected".to_string())
}

#[tauri::command]
async fn monitor_disconnect(state: State<'_, SerialState>) -> Result<String, String> {
    *state.should_run.lock().unwrap() = false;
    *state.port.lock().unwrap() = None;
    println!("Monitor disconnect");
    Ok("Disconnected".to_string())
}

#[tauri::command]
async fn monitor_send(state: State<'_, SerialState>, data: String) -> Result<String, String> {
    let mut guard = state.port.lock().unwrap();
    if let Some(port) = guard.as_mut() {
        let data_bytes = format!("{}\r\n", data); // Add newline for convenience
        port.write_all(data_bytes.as_bytes())
            .map_err(|e| e.to_string())?;
        println!("Monitor send: {}", data);
        Ok("Sent".to_string())
    } else {
        Err("Not connected".to_string())
    }
}

#[tauri::command]
async fn pick_firmware_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    println!("Command 'pick_firmware_file' invoked!");
    use tauri_plugin_dialog::DialogExt;

    println!("Opening dialog...");
    let file_path = app
        .dialog()
        .file()
        .add_filter("Firmware", &["bin"])
        .blocking_pick_file();

    println!("Dialog result: {:?}", file_path);
    Ok(file_path.map(|path| path.to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SerialState {
            port: Arc::new(Mutex::new(None)),
            should_run: Arc::new(Mutex::new(false)),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            check_device_status,
            get_chip_info,
            check_ch34x_driver,
            flash_firmware,
            monitor_connect,
            monitor_disconnect,
            monitor_send,
            pick_firmware_file,
            erase_flash
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
