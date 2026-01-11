use crate::components::{Button, Card};
use crate::i18n::{get_dict, Language};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DeviceStatus {
    code: String, // "ok", "missing_driver", "none"
    message: String,
    port_name: Option<String>,
    product_name: Option<String>,
    serial_number: Option<String>,
    vid_pid: Option<String>,
    connection_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChipDetails {
    chip_model: Option<String>,
    mac_address: Option<String>,
    flash_size: Option<String>,
    chip_revision: Option<String>,
    crystal_frequency: Option<String>,
    features: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct GetChipInfoArgs {
    #[serde(rename = "portName")]
    port_name: String,
}

#[component]
pub fn Home() -> Element {
    let lang = use_context::<Signal<Language>>();
    let dict = get_dict(*lang.read());

    // Default status: disconnected
    let mut device_status = use_signal(|| DeviceStatus {
        code: "none".to_string(),
        message: "Disconnected".to_string(),
        port_name: None,
        product_name: None,
        serial_number: None,
        vid_pid: None,
        connection_type: None,
    });

    let mut chip_details = use_signal(|| ChipDetails {
        chip_model: None,
        mac_address: None,
        flash_size: None,
        chip_revision: None,
        crystal_frequency: None,
        features: None,
        error: None,
    });

    // Manual refresh handler
    let refresh_chip_info = move |_| {
        spawn(async move {
            // Clone port to avoid holding read lock across await
            let port_opt = device_status.read().port_name.clone();

            if let Some(port) = port_opt {
                let args =
                    serde_wasm_bindgen::to_value(&GetChipInfoArgs { port_name: port }).unwrap();

                match invoke("get_chip_info", args).await {
                    Ok(detail_res) => {
                        if let Ok(details) =
                            serde_wasm_bindgen::from_value::<ChipDetails>(detail_res)
                        {
                            chip_details.set(details);
                        }
                    }
                    Err(e) => {
                        chip_details.write().error = Some(format!("Invoke Error: {:?}", e));
                    }
                }
            }
        });
    };

    // Driver check handler
    let mut driver_status = use_signal(|| Option::<bool>::None);
    let check_driver = move |_: MouseEvent| {
        spawn(async move {
            match invoke("check_ch34x_driver", JsValue::NULL).await {
                Ok(res) => {
                    if let Some(is_installed) = res.as_bool() {
                        driver_status.set(Some(is_installed));
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&e);
                }
            }
        });
    };

    // Polling effect (every 2s)
    use_effect(move || {
        spawn(async move {
            loop {
                // Manually call check
                match invoke("check_device_status", JsValue::NULL).await {
                    Ok(js_res) => {
                        if let Ok(res) = serde_wasm_bindgen::from_value::<DeviceStatus>(js_res) {
                            let current_code = device_status.read().code.clone();
                            let current_port = device_status.read().port_name.clone();
                            device_status.set(res.clone());

                            // Trigger chip info fetch only if connected and not yet fetched
                            // Or if port changed
                            if res.code == "ok" {
                                let new_port = res.port_name.clone();
                                // If it's a new connection or we haven't fetched details yet
                                if current_code != "ok" || current_port != new_port {
                                    // Clear previous details
                                    chip_details.set(ChipDetails {
                                        chip_model: None,
                                        mac_address: None,
                                        flash_size: None,
                                        chip_revision: None,
                                        crystal_frequency: None,
                                        features: None,
                                        error: None,
                                    });

                                    // AUTO-FETCH with Retry
                                    let port_clone = new_port.clone();
                                    if let Some(port) = port_clone {
                                        spawn(async move {
                                            let args =
                                                serde_wasm_bindgen::to_value(&GetChipInfoArgs {
                                                    port_name: port,
                                                })
                                                .unwrap();
                                            match invoke("get_chip_info", args).await {
                                                Ok(detail_res) => {
                                                    if let Ok(details) =
                                                        serde_wasm_bindgen::from_value::<ChipDetails>(
                                                            detail_res,
                                                        )
                                                    {
                                                        chip_details.set(details);
                                                    }
                                                }
                                                Err(e) => {
                                                    chip_details.write().error =
                                                        Some(format!("Error: {:?}", e));
                                                }
                                            }
                                        });
                                    }
                                }
                            } else {
                                // Clear details if disconnected
                                if current_code == "ok" {
                                    chip_details.set(ChipDetails {
                                        chip_model: None,
                                        mac_address: None,
                                        flash_size: None,
                                        chip_revision: None,
                                        crystal_frequency: None,
                                        features: None,
                                        error: None,
                                    });
                                }
                            }
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(&e);
                    }
                }
                // Sleep 2s
                gloo_timers::future::TimeoutFuture::new(2000).await;
            }
        });
    });

    rsx! {
        div {
            class: "dashboard-container",
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(350px, 1fr)); gap: 24px;",

            // Card 1: Device Status
            Card {
                title: dict.device_status_title.to_string(),
                subtitle: if let Some(model) = &chip_details.read().chip_model {
                    format!("{} Connected", model)
                } else if let Some(product) = &device_status.read().product_name {
                        product.clone()
                } else {
                    dict.device_status_subtitle.to_string()
                },
                actions: rsx! {
                        if device_status.read().code == "missing_driver" {
                            Button {
                                variant: "tonal".to_string(),
                                icon: "download".to_string(),
                                "Install Driver"
                            }
                        }
                    // Driver Check Button (When disconnected)
                    if device_status.read().code == "none" {
                            Button {
                                variant: "text".to_string(),
                                icon: "verified".to_string(), // or 'security' or 'build'
                                onclick: check_driver,
                                "{dict.driver_check_btn}"
                            }
                    }
                    // Refresh Button (Manual Trigger for Level 2 Info)
                    if device_status.read().code == "ok" {
                        Button {
                            variant: "text".to_string(),
                            icon: "refresh".to_string(),
                            onclick: refresh_chip_info,
                            // No label, just icon? Or "Refresh"? User said "Refresh Button"
                            // Let's use icon only or minimal text if needed.
                            // Given "changed to refresh button", usually implies icon.
                        }
                    }
                },
                div {
                    style: "display: flex; flex-direction: column; gap: 16px; margin-top: 16px;",

                    // Connection Status Row
                    div {
                            style: "display: flex; align-items: center; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--md-sys-color-outline-variant);",
                            span {
                                class: "material-symbols-outlined",
                                style: if device_status.read().code == "ok" { "color: var(--md-sys-color-green, #4caf50); font-size: 24px;" }
                                    else if device_status.read().code == "missing_driver" { "color: var(--md-sys-color-warning, #ffC107); font-size: 24px;" }
                                    else { "color: var(--md-sys-color-error); font-size: 24px;" },
                                if device_status.read().code == "ok" { "check_circle" }
                                else if device_status.read().code == "missing_driver" { "warning" }
                                else { "error" }
                            }
                            div {
                                style: "display: flex; flex-direction: column;",
                                span {
                                    style: "font-weight: 500; color: var(--md-sys-color-on-surface);",
                                    "{device_status.read().message}"
                                }
                                if device_status.read().code == "ok" {
                                    span {
                                        style: "font-size: 0.8em; color: var(--md-sys-color-on-surface-variant);",
                                        "{dict.ready_to_flash}"
                                    }
                                }
                            }
                    }

                    // Error Row (if probing failed)
                    if let Some(err) = &chip_details.read().error {
                        div {
                            style: "background-color: var(--md-sys-color-error-container); color: var(--md-sys-color-on-error-container); padding: 8px 12px; border-radius: 8px; font-size: 0.9em; display: flex; gap: 8px; align-items: center;",
                            span { class: "material-symbols-outlined", style: "font-size: 18px;", "report" }
                            "{dict.probing_error}: {err}"
                        }
                    }

                    // Driver Status (Result of manual check)
                    if let Some(is_installed) = *driver_status.read() {
                            div {
                            style: if is_installed {
                                "background-color: var(--md-sys-color-green-container, #e8f5e9); color: var(--md-sys-color-on-green-container, #1b5e20); padding: 8px 12px; border-radius: 8px; font-size: 0.9em; display: flex; gap: 8px; align-items: center;"
                            } else {
                                "background-color: var(--md-sys-color-error-container); color: var(--md-sys-color-on-error-container); padding: 8px 12px; border-radius: 8px; font-size: 0.9em; display: flex; gap: 8px; align-items: center;"
                            },
                            span {
                                class: "material-symbols-outlined",
                                style: "font-size: 18px;",
                                if is_installed { "check_circle" } else { "warning" }
                            }
                            if is_installed { "{dict.driver_installed}" } else { "{dict.driver_not_found}" }
                        }
                    }

                    // Details Section
                    if device_status.read().code != "none" {
                        // Level 1: Basic Connection Info
                        div {
                            class: "info-section",
                            style: "display: flex; flex-direction: column; gap: 12px;",

                            div {
                                style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 12px;",

                                if let Some(port) = &device_status.read().port_name {
                                    InfoItem {
                                        icon: "usb",
                                        label: dict.port.to_string(),
                                        value: port.clone(),
                                    }
                                }
                                if let Some(vid_pid) = &device_status.read().vid_pid {
                                    InfoItem {
                                        icon: "fingerprint",
                                        label: dict.vid_pid.to_string(),
                                        value: vid_pid.clone(),
                                    }
                                }
                                if let Some(sn) = &device_status.read().serial_number {
                                    if chip_details.read().mac_address.as_ref() != Some(sn) {
                                        InfoItem {
                                            icon: "pin",
                                            label: dict.serial_number.to_string(),
                                            value: sn.clone(),
                                            full_width: true,
                                        }
                                    }
                                }
                                if let Some(ctype) = &device_status.read().connection_type {
                                    InfoItem {
                                        icon: "cable",
                                        label: dict.connection_type.to_string(),
                                        value: if ctype == "native_usb" { dict.type_native_usb.to_string() } else { dict.type_uart_bridge.to_string() },
                                    }
                                }
                            }

                            // Level 2: Chip Details (Only if available)
                            if chip_details.read().chip_model.is_some() {
                                div {
                                    style: "height: 1px; background-color: var(--md-sys-color-outline-variant); margin: 8px 0;",
                                }
                                div {
                                    style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 12px;",

                                    if let Some(model) = &chip_details.read().chip_model {
                                        InfoItem {
                                            icon: "memory",
                                            label: dict.chip_model.to_string(),
                                            value: model.clone(),
                                        }
                                    }
                                    if let Some(flash) = &chip_details.read().flash_size {
                                        InfoItem {
                                            icon: "save",
                                            label: dict.flash_size.to_string(),
                                            value: flash.clone(),
                                        }
                                    }
                                    if let Some(mac) = &chip_details.read().mac_address {
                                        InfoItem {
                                            icon: "lan",
                                            label: dict.mac_address.to_string(),
                                            value: mac.clone(),
                                            full_width: true,
                                        }
                                    }
                                    if let Some(rev) = &chip_details.read().chip_revision {
                                        InfoItem {
                                            icon: "verified_user",
                                            label: dict.chip_revision.to_string(),
                                            value: rev.clone(),
                                        }
                                    }
                                    if let Some(freq) = &chip_details.read().crystal_frequency {
                                        InfoItem {
                                            icon: "sensors",
                                            label: dict.crystal_frequency.to_string(),
                                            value: freq.clone(),
                                        }
                                    }
                                    if let Some(feats) = &chip_details.read().features {
                                        InfoItem {
                                            icon: "featured_play_list",
                                            label: dict.features.to_string(),
                                            value: feats.clone(),
                                            full_width: true,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InfoItem(
    icon: String,
    label: String,
    value: String,
    #[props(default = false)] full_width: bool,
) -> Element {
    let grid_column = if full_width { "span 2" } else { "auto" };

    rsx! {
        div {
            style: "grid-column: {grid_column}; background: var(--md-sys-color-surface-container); padding: 12px; border-radius: 12px; display: flex; align-items: center; gap: 12px; transition: background 0.2s;",

            // Icon container
            div {
                style: "width: 32px; height: 32px; border-radius: 8px; background: var(--md-sys-color-secondary-container); color: var(--md-sys-color-on-secondary-container); display: flex; align-items: center; justify-content: center;",
                span { class: "material-symbols-outlined", style: "font-size: 20px;", "{icon}" }
            }

            // Text content
            div {
                style: "display: flex; flex-direction: column; overflow: hidden;",
                span {
                    style: "font-size: 0.75em; color: var(--md-sys-color-on-surface-variant); white-space: nowrap; text-overflow: ellipsis; overflow: hidden;",
                    "{label}"
                }
                span {
                    style: "font-weight: 500; font-size: 0.9em; color: var(--md-sys-color-on-surface); white-space: nowrap; text-overflow: ellipsis; overflow: hidden;",
                    title: "{value}",
                    "{value}"
                }
            }
        }
    }
}
