#![allow(dead_code)]
use wide::f32x16;

use crate::{interval::{Interval, IntervalS}, vec::Vec3, vec_simd::Vec3S};

pub type Color = Vec3;
pub type ColorS = Vec3S;

const INTENSITY: Interval = Interval {
    min: 0.0,
    max: 0.999,
};

const INTENSITYS: IntervalS = IntervalS {
    min: f32x16::splat(0.0),
    max: f32x16::splat(0.999),
};

#[inline]
fn linear_to_gamma(linear_component: f32) -> f32 {
    if linear_component > 0.0 {
        return linear_component.sqrt();
    }
    0.0
}

#[inline]
fn linear_to_gamma_simd(linear_component: f32x16) -> f32x16 {
    let ret = linear_component.is_sign_positive().select(
        linear_component.sqrt(), 
        linear_component
    );
    ret
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

