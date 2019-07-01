#[macro_use]
extern crate log;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init() {
    // this will trigger `console.error` to be called during panics
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
}

#[wasm_bindgen]
pub fn hello() {
    info!("{}", "Hello, world!");
}
