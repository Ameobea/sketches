extern crate common;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init() {
    common::set_panic_hook(); // this will trigger `console.error` to be called during panics
}

#[wasm_bindgen]
pub fn hello() {
    common::log("Hello, world!")
}
