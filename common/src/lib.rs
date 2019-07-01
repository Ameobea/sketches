#![feature(box_syntax)]

extern crate uuid;
extern crate wasm_bindgen;

use std::fmt::Debug;
use std::mem;

use uuid::Uuid;
use wasm_bindgen::prelude::*;

pub mod color;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = Math)]
    pub fn random() -> f64;
}

pub fn debug<T: Debug>(x: T) -> String {
    format!("{:?}", x)
}

pub fn math_random() -> f64 {
    random()
}

/// Simulates a random UUID, but uses the rand crate with WebAssembly support.
pub fn v4_uuid() -> Uuid {
    // Because I really don't care, honestly.
    let high_quality_entropy: (f64, f64) = (math_random(), math_random());
    unsafe { mem::transmute(high_quality_entropy) }
}
