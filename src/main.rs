pub mod app;
pub mod components;
pub mod i18n;
pub mod pages;

use app::App;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    // Init panic hook to avoid Dioxus overlay crashes
    console_error_panic_hook::set_once();

    launch(App);
}
