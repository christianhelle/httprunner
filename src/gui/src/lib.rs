#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod environment_editor;
#[cfg(target_arch = "wasm32")]
mod file_tree;
#[cfg(target_arch = "wasm32")]
mod request_editor;
#[cfg(target_arch = "wasm32")]
mod results_view;
#[cfg(target_arch = "wasm32")]
mod results_view_async;
#[cfg(target_arch = "wasm32")]
mod state;
#[cfg(target_arch = "wasm32")]
mod text_editor;

#[cfg(target_arch = "wasm32")]
use dioxus::LaunchBuilder;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    LaunchBuilder::new().launch(app::app);
    Ok(())
}
