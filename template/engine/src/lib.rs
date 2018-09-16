extern crate common;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hello() {
    common::log("Hello, world!")
}
