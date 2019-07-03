use std::f32;

pub fn sigmoid(x: f32) -> f32 {
    1. / (1. + f32::consts::E.powf(-x))
}
