#![feature(const_transmute, thread_local)]

use std::mem;

use rand::Rng;
use rand_pcg::Pcg32;

pub mod math;

#[thread_local]
static mut RNG: Pcg32 = unsafe { mem::transmute(0u128) };

pub fn reinit_rng(seed: Option<u128>) {
    *rng() = seed
        .map(|seed| unsafe { mem::transmute(seed) })
        .unwrap_or(Pcg32::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7));
    rng().gen::<u32>();
    rng().gen::<u32>();
}

pub fn rng() -> &'static mut Pcg32 {
    unsafe { &mut RNG }
}
