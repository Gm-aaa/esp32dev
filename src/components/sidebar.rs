use crate::app::Route;
use crate::i18n::{get_dict, Language};
use dioxus::prelude::*;

#[component]
pub fn Sidebar(
    on_theme_toggle: Option<EventHandler<MouseEvent>>,
    on_lang_toggle: Option<EventHandler<MouseEvent>>,
    is_dark: bool,
) -> Element {
    let theme_icon = if is_dark { "light_mode" } else { "dark_mode" };
    let current_route = use_route::<Route>();
    let lang = use_context::<Signal<Language>>();
    let dict = get_dict(*lang.read());

    rsx! {
        div {
            class: "md-sidebar",
            NavItem {
                icon: "home".to_string(),
                label: dict.home_nav.to_string(),
                to: Route::Home {},
                active: current_route == Route::Home {},
            }
            NavItem {
                icon: "developer_board".to_string(),
                label: dict.devices_nav.to_string(),
                to: Route::Devices {},
                active: current_route == Route::Devices {},
            }

            // Spacer
            div { style: "flex: 1;" }

            // Bottom Actions
            div {
                class: "md-nav-item",
                onclick: move |evt| if let Some(h) = &on_lang_toggle { h.call(evt) },
                span { class: "material-symbols-outlined icon", "language" }
            }
            div {
                class: "md-nav-item",
                onclick: move |evt| if let Some(h) = &on_theme_toggle { h.call(evt) },
                span { class: "material-symbols-outlined icon", "{theme_icon}" }
            }

            div { style: "height: 12px;" } // Bottom padding
        }
    }
}

#[component]
fn NavItem(
    icon: String,
    label: String,
    to: Route,
    #[props(default = false)] active: bool,
) -> Element {
    let active_class = if active { "active" } else { "" };

    rsx! {
        Link {
            to: to,
            class: "md-nav-item {active_class}",
            span { class: "material-symbols-outlined icon", "{icon}" }
            "{label}"
        }
    }
}
