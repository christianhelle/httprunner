#![allow(dead_code)]

mod app;
mod environment_editor;
mod file_tree;
mod request_editor;
mod request_view;
mod results_view;
mod state;
mod text_editor;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    dioxus::launch(app::App);
}
