use crate::components::sidebar::Sidebar;
use dioxus::prelude::*;

#[component]
pub fn Layout(
    children: Element,
    on_theme_toggle: Option<EventHandler<MouseEvent>>,
    on_lang_toggle: Option<EventHandler<MouseEvent>>,
    is_dark: bool,
) -> Element {
    rsx! {
        div {
            class: "md-layout",
            Sidebar {
                on_theme_toggle: on_theme_toggle,
                on_lang_toggle: on_lang_toggle,
                is_dark: is_dark
            }
            main {
                class: "md-main-content",
                {children}
            }
        }
    }
}
