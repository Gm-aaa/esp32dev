#![allow(non_snake_case)]

use crate::components::Layout;
use crate::i18n::Language;
use crate::pages::devices::Devices;
use crate::pages::home::Home;
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

static CSS: Asset = asset!("/assets/styles.css");
static MATERIAL_CSS: Asset = asset!("/assets/material.css");

// Helper to switch theme
#[wasm_bindgen(
    inline_js = "export function set_theme(theme) { document.documentElement.setAttribute('data-theme', theme); }"
)]
extern "C" {
    fn set_theme(theme: &str);
}

#[derive(Clone, Copy, PartialEq)]
enum Theme {
    Light,
    Dark,
}

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Home {},
    #[route("/devices")]
    Devices {},
    #[end_layout]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        div { "Page not found: {route:?}" }
    }
}

pub fn App() -> Element {
    rsx! {
        link { rel: "stylesheet", href: CSS }
        link { rel: "stylesheet", href: MATERIAL_CSS }
        Router::<Route> {}
    }
}

#[component]
fn AppLayout() -> Element {
    let mut theme = use_signal(|| Theme::Dark);
    let mut lang = use_context_provider(|| Signal::new(Language::Zh));

    // Apply initial theme
    use_effect(move || {
        set_theme("dark");
    });

    let toggle_theme = move |_| {
        let new_theme = match *theme.read() {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
        theme.set(new_theme);

        let theme_str = match new_theme {
            Theme::Light => "light",
            Theme::Dark => "dark",
        };
        set_theme(theme_str);
    };

    let toggle_lang = move |_| {
        let new_lang = match *lang.read() {
            Language::En => Language::Zh,
            Language::Zh => Language::En,
        };
        lang.set(new_lang);
    };

    rsx! {
        Layout {
            on_theme_toggle: toggle_theme,
            on_lang_toggle: toggle_lang,
            is_dark: *theme.read() == Theme::Dark,
            Outlet::<Route> {}
        }
    }
}
