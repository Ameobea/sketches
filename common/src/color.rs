use std::mem;

use super::math_random;

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Color { red, green, blue }
    }

    pub fn random() -> Self {
        let (red, green, blue, _): (u8, u8, u8, [u8; 5]) = unsafe { mem::transmute(math_random()) };
        Color { red, green, blue }
    }
}
