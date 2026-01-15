use crate::components::{Button, Card, PinoutView};
use crate::i18n::{get_dict, Language};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>)
        -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DeviceStatus {
    code: String,
    message: String,
    port_name: Option<String>,
    product_name: Option<String>,
    serial_number: Option<String>,
    vid_pid: Option<String>,
    connection_type: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FlashArgs {
    port_name: String,
    firmware_path: String,
    flash_address: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MonitorConnectArgs {
    port_name: String,
    baud_rate: u32,
}

#[derive(Serialize)]
struct MonitorSendArgs {
    data: String,
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
pub fn Devices() -> Element {
    // Shared State
    let mut port_name = use_signal(|| "".to_string());

    // Flashing State
    let mut firmware_path = use_signal(|| "".to_string());
    let mut flash_address = use_signal(|| "0x0".to_string());
    let mut is_flashing = use_signal(|| false);
    let mut is_erasing = use_signal(|| false);
    let mut erase_msg = use_signal(|| "".to_string());
    let mut flash_progress = use_signal(|| 0.0);

    // Monitor State
    let mut baud_rate = use_signal(|| "115200".to_string());
    let mut is_connected = use_signal(|| false);
    let mut logs = use_signal(|| Vec::<String>::new()); // Mock logs
    let mut input_cmd = use_signal(|| "".to_string());

    // Tab State
    let mut active_tab = use_signal(|| "monitor".to_string());
    let mut detected_model = use_signal(|| "ESP32-S3".to_string()); // Default or detected
    let mut detected_connection_type = use_signal(|| None::<String>);
    let mut chip_details_info = use_signal(|| None::<ChipDetails>);

    let lang = use_context::<Signal<Language>>();
    let dict = get_dict(*lang.read());

    // Auto-detect port on mount
    use_effect(move || {
        spawn(async move {
            if let Ok(js_res) = invoke("check_device_status", JsValue::NULL).await {
                if let Ok(res) = serde_wasm_bindgen::from_value::<DeviceStatus>(js_res) {
                    if let Some(p) = res.port_name.clone() {
                        port_name.set(p.clone());

                        if let Some(conn_type) = res.connection_type.clone() {
                            detected_connection_type.set(Some(conn_type));
                        }

                        // Optimisation: Fetch real chip info
                        spawn(async move {
                            let args = match serde_wasm_bindgen::to_value(&GetChipInfoArgs {
                                port_name: p,
                            }) {
                                Ok(a) => a,
                                Err(e) => {
                                    web_sys::console::error_1(&e.to_string().into());
                                    return;
                                }
                            };
                            match invoke("get_chip_info", args).await {
                                Ok(val) => {
                                    if let Ok(info) =
                                        serde_wasm_bindgen::from_value::<ChipDetails>(val)
                                    {
                                        if let Some(model) = info.chip_model.clone() {
                                            detected_model.set(model);
                                        }
                                        chip_details_info.set(Some(info));
                                    }
                                }
                                Err(e) => {
                                    web_sys::console::log_1(&e);
                                }
                            }
                        });
                    }
                }
            }
        });
    });

    // Listener cleanup guard
    struct ListenerGuard {
        unlisten: Option<js_sys::Function>,
        _closure: Option<Closure<dyn FnMut(JsValue)>>,
    }
    impl Drop for ListenerGuard {
        fn drop(&mut self) {
            // Unlisten
            if let Some(f) = &self.unlisten {
                web_sys::console::log_1(&"Unlistening serial-read".into());
                f.call0(&JsValue::NULL).ok();
            }

            // Auto-disconnect backend
            spawn(async move {
                let _ = invoke("monitor_disconnect", JsValue::NULL).await;
            });
        }
    }

    // We use a signal to hold the guard so it drops when the component is unmounted
    let mut listener_guard = use_signal(|| {
        Chunk(ListenerGuard {
            unlisten: None,
            _closure: None,
        })
    });
    // Helper wrapper because ListenerGuard doesn't implement Clone/PartialEq which Signal might want,
    // actually Signal<T> just needs T: 'static.
    // To be safe against Dioxus diffing, we wrap in a newtype transparently or just use it.
    // Dioxus 0.5 Signal holds RefCell<T>.
    struct Chunk(ListenerGuard);

    // Listen for serial data
    use_effect(move || {
        spawn(async move {
            let closure = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                #[derive(Deserialize)]
                struct SerialEvent {
                    payload: String,
                }
                if let Ok(e) = serde_wasm_bindgen::from_value::<SerialEvent>(event) {
                    // Check if write is safe? Dioxus panic implies we can't write if dropped.
                    // But if we are here, closure is alive.
                    // If component dropped, signal dropped?
                    // The panic "Result::unwrap() on Err value: Dropped"
                    // implies logs signal is accessed after drop.
                    logs.write().push(e.payload);
                }
            });

            match listen("serial-read", &closure).await {
                Ok(unlisten_js) => {
                    let unlisten = unlisten_js.dyn_into::<js_sys::Function>().ok();
                    // Store both to keep them alive until guard is dropped
                    listener_guard.write().0 = ListenerGuard {
                        unlisten,
                        _closure: Some(closure),
                    };
                }
                Err(e) => {
                    web_sys::console::error_1(&e);
                }
            }
        });
    });

    rsx! {
        div {
            class: "devices-container",
            // Use Flexbox with wrap to allow stacking on small screens
            style: "display: flex; flex-wrap: wrap; gap: 24px; height: 100%; align-items: flex-start; overflow-y: auto; padding-bottom: 24px;",

            // Left: Flashing Panel
            div { style: "flex: 1; min-width: 300px;",
                Card {
                    title: dict.devices_title_flashing.to_string(),
                    subtitle: dict.devices_subtitle_flashing.to_string(),

                    div { style: "display: flex; flex-direction: column; gap: 16px; margin-top: 16px;",

                        // File Selection
                        div {
                            label { r#for: "firmware_path", style: "display: block; font-size: 0.8em; margin-bottom: 4px; color: var(--md-sys-color-on-surface-variant);",
                                "{dict.devices_label_firmware_file}"
                            }
                            div { style: "display: flex; gap: 8px;",
                                input {
                                    r#type: "text",
                                    name: "firmware_path",
                                    id: "firmware_path",
                                    value: "{firmware_path}",
                                    placeholder: "{dict.devices_placeholder_firmware_file}",
                                    class: "md-input",
                                    style: "flex: 1;",
                                    oninput: move |evt| firmware_path.set(evt.value()),
                                }
                                button {
                                    class: "md-button btn-tonal",
                                    onclick: move |_| {
                                        web_sys::console::log_1(&"Browse button clicked".into());
                                        spawn(async move {
                                            match invoke("pick_firmware_file", JsValue::NULL).await {
                                                Ok(res) => {
                                                    web_sys::console::log_1(&"Invoke success".into());
                                                    if let Some(path) = res.as_string() {
                                                        firmware_path.set(path);
                                                    }
                                                }
                                                Err(e) => {
                                                    web_sys::console::error_1(&e);
                                                }
                                            }
                                        });
                                    },
                                    span { class: "material-symbols-outlined icon", "folder_open" }
                                    span { class: "label", "{dict.devices_btn_browse}" }
                                }
                            }
                        }

                        // Address Config
                        div {
                            label { r#for: "flash_address", style: "display: block; font-size: 0.8em; margin-bottom: 4px; color: var(--md-sys-color-on-surface-variant);",
                                "{dict.devices_label_flash_address}"
                            }
                            input {
                                r#type: "text",
                                name: "flash_address",
                                id: "flash_address",
                                value: "{flash_address}",
                                class: "md-input",
                                style: "width: 100%;",
                                oninput: move |evt| flash_address.set(evt.value()),
                            }
                        }

                        // Progress Bar
                        if *is_flashing.read() {
                            div { style: "display: flex; flex-direction: column; gap: 4px;",
                                div { style: "display: flex; justify-content: space-between; font-size: 0.8em;",
                                    span { "{dict.devices_flashing_status}" }
                                    span { "{flash_progress.read()}%" }
                                }
                                div { style: "height: 4px; background: var(--md-sys-color-surface-container-highest); border-radius: 2px; overflow: hidden;",
                                    div { style: "height: 100%; background: var(--md-sys-color-primary); width: {flash_progress.read()}%; transition: width 0.2s;" }
                                }
                            }
                        }

                        // Action Button
                        Button {
                            variant: "filled".to_string(),
                            icon: "bolt".to_string(),
                            onclick: move |_| {
                                let path = firmware_path.read().clone();
                                let addr = flash_address.read().clone();
                                let port = port_name.read().clone(); // Use dynamic port

                                spawn(async move {
                                    if port.is_empty() {
                                        web_sys::console::error_1(&"No port selected".into());
                                        return;
                                    }

                                    is_flashing.set(true);
                                    flash_progress.set(0.0);

                                    let args = serde_wasm_bindgen::to_value(

                                            &FlashArgs {
                                                port_name: port,
                                                firmware_path: path,
                                                flash_address: addr,
                                            },
                                        )
                                        .unwrap();
                                    match invoke("flash_firmware", args).await {
                                        Ok(_) => {
                                            flash_progress.set(100.0);
                                            is_flashing.set(false);
                                        }
                                        Err(e) => {
                                            web_sys::console::error_1(&e);
                                            is_flashing.set(false);
                                        }
                                    }
                                });
                            },
                            "{dict.devices_btn_start_flash}"
                        }

                        // Erase Button
                        Button {
                            variant: "tonal".to_string(),
                            icon: "delete_forever".to_string(),
                            onclick: move |_| {
                                let port = port_name.read().clone();
                                spawn(async move {
                                    if port.is_empty() {
                                        web_sys::console::error_1(&"No port selected".into());
                                        return;
                                    }
                                    is_erasing.set(true);

                                    // FIX 1: Use snake_case "port_name" to match Rust backend

                                    // FIX 2: No alert, just log. Better UX would be a toast or status text.
                                    let args = serde_wasm_bindgen::to_value(&json!({ "portName" : port }))
                                        .unwrap_or(JsValue::NULL);
                                    web_sys::console::log_1(&"Invoking erase_flash...".into());
                                    erase_msg.set("".to_string());
                                    match invoke("erase_flash", args).await {
                                        Ok(_) => {
                                            web_sys::console::log_1(&"Erase success".into());
                                            erase_msg.set("清除成功！".to_string());
                                            // Clear message after 3 seconds
                                            gloo_timers::future::TimeoutFuture::new(3000).await;
                                            erase_msg.set("".to_string());
                                        }
                                        Err(e) => {
                                            web_sys::console::error_1(&e);
                                            erase_msg.set("清除失败！".to_string());
                                        }
                                    }
                                    is_erasing.set(false);
                                });
                            },
                            if *is_erasing.read() {
                                "清除中..."
                            } else {
                                "{dict.devices_btn_erase_flash}"
                            }
                        }
                        if !erase_msg.read().is_empty() {
                            div { style: "font-size: 0.8em; margin-top: 4px; color: var(--md-sys-color-primary);",
                                "{erase_msg}"
                            }
                        }
                    }
                }
            }

            // Right: Tabbed Panel
            div { style: "flex: 1.5; min-width: 350px; display: flex; flex-direction: column; gap: 12px;",

                // Tabs
                div { style: "display: flex; gap: 8px; border-bottom: 1px solid var(--md-sys-color-outline-variant); padding-bottom: 8px;",

                    button {
                        class: if *active_tab.read() == "monitor" { "md-button btn-tonal" } else { "md-button btn-text" },
                        style: "border-radius: 8px 8px 0 0;",
                        onclick: move |_| active_tab.set("monitor".to_string()),
                        span { class: "material-symbols-outlined icon", "terminal" }
                        "{dict.monitor_tab}"
                    }
                    button {
                        class: if *active_tab.read() == "pinout" { "md-button btn-tonal" } else { "md-button btn-text" },
                        style: "border-radius: 8px 8px 0 0;",
                        onclick: move |_| active_tab.set("pinout".to_string()),
                        span { class: "material-symbols-outlined icon", "developer_board" }
                        "{dict.board_view_tab}"
                    }
                }

                if *active_tab.read() == "monitor" {
                    Card {
                        title: dict.devices_title_monitor.to_string(),
                        subtitle: dict.devices_subtitle_monitor.to_string(),
                        actions: rsx! {
                            // Port Input
                            // Baud Rate Select









                            div { style: "display: flex; align-items: center; gap: 8px;",
                                span {
                                    style: "font-size: 0.9em; color: var(--md-sys-color-on-surface-variant);",
                                    "Port" // TODO: Add to Dict
                                }
                                input {
                                    r#type: "text",
                                    name: "monitor_port",
                                    id: "monitor_port",
                                    value: "{port_name}",
                                    class: "md-input",
                                    style: "width: 80px;",
                                    oninput: move |evt| port_name.set(evt.value()),
                                }
                            }
                            div { style: "display: flex; align-items: center; gap: 8px; margin-right: 8px;",
                                span { style: "font-size: 0.9em; color: var(--md-sys-color-on-surface-variant);",
                                    label {
                                        r#for: "baud_rate",
                                        "{dict.devices_label_baud_rate}"
                                    }
                                }
                                select {
                                    class: "md-select",
                                    name: "baud_rate",
                                    id: "baud_rate",
                                    value: "{baud_rate}",
                                    onchange: move |evt| baud_rate.set(evt.value()),
                                    option { value: "9600", "9600" }
                                    option { value: "115200", "115200" }
                                    option { value: "921600", "921600" }
                                }
                            }
                            Button {
                                variant: "text".to_string(),
                                icon: "delete_sweep".to_string(),
                                onclick: move |_| {
                                    logs.write().clear();
                                },
                                "{dict.devices_btn_clear}"
                            }
                            Button {
                                variant: { if *is_connected.read() { "tonal" } else { "text" } }.to_string(),
                                icon: { if *is_connected.read() { "link_off" } else { "link" } }.to_string(),
                                onclick: move |_| {
                                    let connected = *is_connected.read();
                                    let port = port_name.read().clone(); // Use dynamic port
                                    let baud_str = baud_rate.read().clone();
                                    let baud = baud_str.parse::<u32>().unwrap_or(115200);

                                    spawn(async move {
                                        if connected {
                                            if invoke("monitor_disconnect", JsValue::NULL).await.is_ok() {
                                                is_connected.set(false);
                                            }
                                        } else {
                                            if port.is_empty() {
                                                web_sys::console::error_1(&"No port selected".into());
                                                return;
                                            }
                                            let args = serde_wasm_bindgen::to_value(

                                                    &MonitorConnectArgs {
                                                        port_name: port,
                                                        baud_rate: baud,
                                                    },
                                                )
                                                .unwrap();
                                            if invoke("monitor_connect", args).await.is_ok() {
                                                is_connected.set(true);
                                            }
                                        }
                                    });
                                },
                                if *is_connected.read() {
                                    "{dict.devices_btn_disconnect}"
                                } else {
                                    "{dict.connect}"
                                }
                            }
                        },

                        div { style: "display: flex; flex-direction: column; gap: 12px; margin-top: 8px;",

                            // Log Area
                            div { style: "background: #1e1e1e; color: #d4d4d4; font-family: 'JetBrains Mono', 'Consolas', 'Courier New', monospace; font-size: 0.9em; padding: 12px; border-radius: 8px; height: 400px; overflow-y: auto; white-space: pre-wrap; word-wrap: break-word;",
                                if logs.read().is_empty() {
                                    span { style: "color: #666;", "{dict.devices_log_placeholder}" }
                                }
                                for log in logs.read().iter() {
                                    span { "{log}" }
                                }
                            }

                            // Input Area
                            div { style: "display: flex; gap: 8px;",
                                input {
                                    r#type: "text",
                                    name: "monitor_input",
                                    id: "monitor_input",
                                    value: "{input_cmd}",
                                    placeholder: "{dict.devices_input_placeholder}",
                                    class: "md-input",
                                    style: "flex: 1;",
                                    oninput: move |evt| input_cmd.set(evt.value()),
                                    onkeypress: move |evt| {
                                        if evt.key() == Key::Enter {
                                            if !input_cmd.read().is_empty() {
                                                logs.write().push(format!("> {}", input_cmd.read()));
                                                input_cmd.set("".to_string());
                                            }
                                        }
                                    },
                                }
                                Button {
                                    variant: "tonal".to_string(),
                                    icon: "send".to_string(),
                                    onclick: move |_| {
                                        let cmd = input_cmd.read().clone();
                                        if !cmd.is_empty() {
                                            logs.write().push(format!("> {}", cmd));
                                            input_cmd.set("".to_string());

                                            spawn(async move {
                                                let args = serde_wasm_bindgen::to_value(&MonitorSendArgs { data: cmd })
                                                    .unwrap();
                                                invoke("monitor_send", args).await.ok();
                                            });
                                        }
                                    },
                                }
                            }
                        }
                    }
                } else {
                    Card {
                        title: dict.board_view_title.to_string(),
                        subtitle: format!("View for {}", detected_model),
                        PinoutView {
                            chip_model: detected_model.read().clone(),
                            connection_type: detected_connection_type.read().clone(),
                        }
                    }
                }
            }

        }
    }
}
