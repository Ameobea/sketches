#![feature(
    box_syntax,
    const_raw_ptr_deref,
    const_fn_union,
    const_transmute,
    thread_local,
    bind_by_move_pattern_guards,
    nll
)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::mem;
use std::panic;

use wasm_bindgen::prelude::*;

pub mod conf;
use self::conf::*;
pub mod minutiae;
pub mod util;

#[wasm_bindgen]
pub fn init() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    set_active_conf(Conf {});
    crate::minutiae::init_universe();
    let seed = unsafe { mem::transmute((common::math_random(), common::math_random())) };
    sketches_util::reinit_rng(Some(seed));
}

#[wasm_bindgen]
pub fn set_user_conf(conf_json: String) {
    let conf: Conf = match serde_json::from_str(&conf_json) {
        Ok(conf) => conf,
        Err(err) => {
            error!("Error parsing provided user conf JSON: {:?}", err);
            return;
        }
    };

    set_active_conf(conf);
}
