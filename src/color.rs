#![allow(dead_code)]
use crate::{interval::Interval, vec::Vec3};

pub type Color = Vec3;

const INTENSITY: Interval = Interval {
    min: 0.0,
    max: 0.999,
};

#[inline]
fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        return linear_component.sqrt();
    }
    0.0
}

pub fn write_color(pixel_color: &Color) {
    let r = linear_to_gamma(pixel_color.x);
    let g = linear_to_gamma(pixel_color.y);
    let b = linear_to_gamma(pixel_color.z);

    // Trnaslate the [0, 1] component values to the byte range [0, 255]
    let rbyte = (INTENSITY.clamp(r) * 256.0) as u16;
    let gbyte = (INTENSITY.clamp(g) * 256.0) as u16;
    let bbyte = (INTENSITY.clamp(b) * 256.0) as u16;

    println!("{} {} {}", rbyte, gbyte, bbyte);
}
