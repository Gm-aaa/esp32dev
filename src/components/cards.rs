use dioxus::prelude::*;

#[component]
pub fn Card(
    #[props(default = "".to_string())] title: String,
    #[props(default = "".to_string())] subtitle: String,
    children: Element,
    actions: Option<Element>,
) -> Element {
    rsx! {
        div {
            class: "md-card",
            if !title.is_empty() {
                div {
                   class: "md-card-header",
                   div { class: "md-card-title", "{title}" }
                   if !subtitle.is_empty() {
                       div { class: "md-card-subtitle", "{subtitle}" }
                   }
                }
            }
            div {
                class: "md-card-content",
                {children}
            }
            if let Some(actions) = actions {
                div {
                    class: "md-card-actions",
                    {actions}
                }
            }
        }
    }
}
