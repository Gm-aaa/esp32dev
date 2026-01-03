use dioxus::prelude::*;

#[component]
pub fn Button(
    #[props(default = "filled".to_string())] variant: String,
    #[props(default = "".to_string())] icon: String,
    children: Element,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let variant_class = match variant.as_str() {
        "tonal" => "btn-tonal",
        "outlined" => "btn-outlined",
        "text" => "btn-text",
        _ => "btn-filled",
    };

    rsx! {
        button {
            class: "md-button {variant_class}",
            onclick: move |evt| if let Some(h) = &onclick { h.call(evt) },
            if !icon.is_empty() {
                span { class: "material-symbols-outlined icon", "{icon}" }
            }
            span { class: "label", {children} }
        }
    }
}
