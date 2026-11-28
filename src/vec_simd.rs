#![allow(dead_code)]

use rand;
use std::{array, fmt, ops};
use std::array::from_fn;
use wide::{Select, f32x16};

use crate::utility::{PI, random_double_range};
use crate::vec::Vec3;

#[derive(Clone, Copy)]
pub struct Vec3S {
    pub x: f32x16,
    pub y: f32x16,
    pub z: f32x16,
}
pub type PointS = Vec3S;

impl Select<Vec3S> for f32x16 {
    fn select(self, if_true: Vec3S, if_false: Vec3S) -> Vec3S {
        Vec3S {
            x: self.select(if_true.x, if_false.x),
            y: self.select(if_true.y, if_false.y),
            z: self.select(if_true.z, if_false.z)
        }
    }
}

impl Vec3S {
    pub fn new(x: f32x16, y: f32x16, z: f32x16) -> Self {
        Self { x, y, z }
    }
    pub fn from_vec(v: Vec3) -> Self {
        let x: [f32; 16] = [v.x; 16];
        let y: [f32; 16] = [v.y; 16];
        let z: [f32; 16] = [v.z; 16];
        Self {
            x: f32x16::new(x),
            y: f32x16::new(y),
            z: f32x16::new(z)
        }
    }
    pub fn to_arr(&self) -> [Vec3; 16] {
        let xs = self.x.to_array();
        let ys = self.y.to_array();
        let zs = self.z.to_array();
        let mut ret = [Vec3::new_zero(); 16];
        for i in 0..16 {
            ret[i] = Vec3::new(xs[i], ys[i], zs[i]);
        }
        ret
    }
    pub fn new_zero() -> Self {
        Self {
            x: f32x16::new([0.0; 16]),
            y: f32x16::new([0.0; 16]),
            z: f32x16::new([0.0; 16]),
        }
    }
    pub fn random() -> Self {
        let x: [f32; 16] = array::from_fn(|_| rand::random::<f32>());
        let y: [f32; 16] = array::from_fn(|_| rand::random::<f32>());
        let z: [f32; 16] = array::from_fn(|_| rand::random::<f32>());
        Vec3S {
            x: f32x16::new(x),
            y: f32x16::new(y),
            z: f32x16::new(z),
        }
    }
    pub fn random_range(min: f32, max: f32) -> Self {
        let x: [f32; 16] = array::from_fn(|_| rand::random_range(min..max));
        let y: [f32; 16] = array::from_fn(|_| rand::random_range(min..max));
        let z: [f32; 16] = array::from_fn(|_| rand::random_range(min..max));
        Vec3S {
            x: f32x16::new(x),
            y: f32x16::new(y),
            z: f32x16::new(z),
        }
    }
    pub fn random_unit() -> Self {
        let z = f32x16::new(array::from_fn(|_| rand::random_range(-1.0..=1.0)));
        let theta = f32x16::new(array::from_fn(|_| rand::random_range(0.0..=(2.0*PI) as f32)));
        let rxy = (1.0 - z * z).sqrt();
        
        Self {
            x: rxy * theta.cos(),
            y: rxy * theta.sin(),
            z
        }
    }
    pub fn random_on_hemisphere(normal: Self) -> Self {
        let on_unit_sphere = Self::random_unit();
        on_unit_sphere.dot(normal).is_sign_positive().select(
            on_unit_sphere,
            on_unit_sphere * f32x16::splat(-1.0),
        )
    }

    pub fn random_in_unit_disk() -> Self {
        Self {
            x: f32x16::new(array::from_fn(|_| rand::random_range(0.0..=(2.0*PI) as f32))),
            y: f32x16::new(array::from_fn(|_| rand::random_range(0.0..=(2.0*PI) as f32))),
            z: f32x16::splat(0.0),
        }
    }

    pub fn reflect(v: Self, n: Self) -> Self {
        v - (n * v.dot(n)) * f32x16::splat(2.0)
    }

    pub fn refract(uv: Self, n: Self, etai_over_etat: f32x16) -> Self {
        let cos_theta = n.dot(uv * f32x16::splat(-1.0)).min(f32x16::splat(1.0));
        let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
        let r_out_perp_len = r_out_perp.length();
        let r_out_parallel = n * (-1.0 * ((1.0 - r_out_perp_len * r_out_perp_len).abs()).sqrt());
        r_out_perp + r_out_parallel
    }
}

impl fmt::Display for Vec3S {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{},{})", self.x, self.y, self.z)
    }
}

impl ops::Add for Vec3S {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl ops::AddAssign for Vec3S {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl ops::AddAssign<&Self> for Vec3S {
    fn add_assign(&mut self, other: &Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl ops::Sub for Vec3S {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl ops::SubAssign for Vec3S {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl ops::SubAssign<&Self> for Vec3S {
    fn sub_assign(&mut self, other: &Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl ops::Mul<Vec3S> for Vec3S {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl ops::Mul<f32x16> for Vec3S {
    type Output = Self;

    fn mul(self, other: f32x16) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl ops::MulAssign for Vec3S {
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
    }
}

impl ops::MulAssign<&Self> for Vec3S {
    fn mul_assign(&mut self, other: &Self) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
    }
}

impl ops::Div<f32x16> for Vec3S {
    type Output = Self;

    fn div(self, other: f32x16) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl ops::DivAssign<f32x16> for Vec3S {
    fn div_assign(&mut self, other: f32x16) {
        self.x /= other;
        self.y /= other;
        self.z /= other;
    }
}

impl Vec3S {
    pub fn length(&self) -> f32x16 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn dot(&self, other: Self) -> f32x16 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn unit(&self) -> Self {
        let len = self.length();
        Self {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }

    pub fn near_zero(&self) -> f32x16 {
        let s = f32x16::splat(1.0f32.powi(-8));
        self.x.abs().simd_lt(s) & self.y.abs().simd_lt(s) & self.z.abs().simd_lt(s)
    }
}
