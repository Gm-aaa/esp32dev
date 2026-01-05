use dioxus::prelude::*;

#[component]
pub fn PinoutView(chip_model: String, connection_type: Option<String>) -> Element {
    // Normalize chip model string for matching
    let model_upper = chip_model.to_uppercase();

    let svg_filename = if model_upper.contains("S3") {
        "esp32-s3.svg"
    } else if model_upper.contains("C3") {
        "esp32-c3.svg"
    } else if model_upper.contains("S2") {
        "esp32-s2.svg"
    } else if model_upper.contains("ESP32") {
        "esp32-s3.svg"
    } else {
        "esp32-s3.svg"
    };

    // Construct absolute path using window origin to avoid "RelativeUrlWithoutBase" error
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "http://localhost:1420".to_string()); // Fallback for dev

    let svg_path = format!("{}/boards/{}", origin, svg_filename);

    // State to hold the fetched SVG content
    let mut svg_content = use_signal(|| "".to_string());
    use_resource(move || {
        let path = svg_path.clone();
        async move {
            web_sys::console::log_1(&format!("Fetching SVG from: {}", path).into());
            match reqwest::get(&path).await {
                Ok(response) => {
                    web_sys::console::log_1(&format!("Fetch status: {}", response.status()).into());
                    match response.text().await {
                        Ok(text) => {
                            web_sys::console::log_1(
                                &format!("SVG content length: {}", text.len()).into(),
                            );
                            svg_content.set(text);
                        }
                        Err(e) => web_sys::console::error_1(
                            &format!("Failed to read text: {:?}", e).into(),
                        ),
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to fetch SVG: {:?}", e).into())
                }
            }
        }
    });

    let mut css_rules =
        "#pinout-container svg { width: 100%; height: 100%; object-fit: contain; } ".to_string();

    if let Some(conn) = connection_type {
        if conn == "native_usb" {
            css_rules.push_str("#USB rect { fill: #4caf50 !important; stroke: #81c784 !important; stroke-width: 2px; } #USB text { fill: #4caf50 !important; font-weight: bold; }");
        } else {
            css_rules.push_str("#COM rect { fill: #4caf50 !important; stroke: #81c784 !important; stroke-width: 2px; } #COM text { fill: #4caf50 !important; font-weight: bold; }");
        }
    }

    rsx! {
        div {
            style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #1e1e1e; border-radius: 8px; overflow: hidden; position: relative;",

            // Inject dynamic styles for highlighting and sizing
            style { "{css_rules}" }

            div {
                style: "width: 100%; height: 100%; padding: 16px; box-sizing: border-box; display: flex; justify-content: center;",
                // Render SVG string
                div {
                    id: "pinout-container",
                    dangerous_inner_html: "{svg_content}",
                    style: "width: 100%; height: 100%; display: flex; justify-content: center; align-items: center;"
                }
            }
        }
    }
}
