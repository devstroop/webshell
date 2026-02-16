//! WebShell Frontend - Leptos-based terminal UI

mod app;
mod terminal;
mod ws;
mod xterm;

use wasm_bindgen::prelude::*;

pub use app::App;

/// Initialize the frontend
#[wasm_bindgen(start)]
pub fn main() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logging
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");

    log::info!("WebShell frontend starting...");

    // Mount the Leptos app
    leptos::mount_to_body(App);
}
