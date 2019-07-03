#![feature(const_transmute, thread_local)]

use std::mem;

use rand::Rng;
use rand_pcg::Pcg32;

pub mod math;

#[thread_local]
static mut RNG: Pcg32 = unsafe { mem::transmute(0u128) };

pub fn reinit_rng() {
    *rng() = unsafe { mem::transmute((-42234i32, 1991u32, -234i32, 44444u32)) };
    rng().gen::<u32>();
    rng().gen::<u32>();
}

pub fn rng() -> &'static mut Pcg32 {
    unsafe { &mut RNG }
}
