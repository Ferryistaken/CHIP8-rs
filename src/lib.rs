pub mod chip8;

#[cfg(feature = "cli")]
pub mod platform;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "web")]
pub mod platform_web;
#[cfg(feature = "web")]
#[wasm_bindgen(start)]
pub fn _set_panic_hook() {
    console_error_panic_hook::set_once();
}
