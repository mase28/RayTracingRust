#![allow(dead_code)]

use rand;
use wide::f32x16;

pub const INF: f32 = f32::MAX;
pub const PI: f64 = 3.1415926535897932385;

#[inline]
pub fn degrees_to_radians(degrees: f32) -> f32 {
    (degrees as f64 * PI / 180.0) as f32
}

#[inline]
pub fn degrees_to_radians_simd(degrees: f32x16) -> f32x16 {
    degrees * PI as f32 / 180.0
}

#[inline]
pub fn random_double() -> f32 {
    rand::random_range(0.0..1.0)
}

#[inline]
pub fn random_double_range(min: f32, max: f32) -> f32 {
    rand::random_range(min..max)
}
