#![allow(dead_code)]

use wide::{Select, f32x16};

use crate::{vec::{Point, Vec3}, vec_simd::{PointS, Vec3S}};

pub struct Ray {
    origin: Point,
    direction: Vec3,
}

#[derive(Clone, Copy)]
pub struct RayS {
    origin: PointS,
    direction: Vec3S,
}

impl Ray {
    pub fn new(origin: Point, direction: Vec3) -> Self {
        Self {
            origin: origin,
            direction: direction,
        }
    }

    pub fn default() -> Self {
        Self {
            origin: Vec3::new_zero(),
            direction: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn origin(&self) -> Point {
        self.origin
    }

    pub fn direction(&self) -> &Vec3 {
        &self.direction
    }

    pub fn at(&self, t: f32) -> Point {
        self.origin + (self.direction * t)
    }
}

impl Select<RayS> for f32x16 {
    fn select(self, if_true: RayS, if_false: RayS) -> RayS {
        RayS {
            origin: self.select(if_true.origin, if_false.origin),
            direction: self.select(if_true.origin, if_false.origin)
        }
    }
}

impl RayS {
    pub fn new(origin: PointS, direction: Vec3S) -> Self {
        Self {
            origin,
            direction,
        }
    }
    pub fn default() -> Self {
        Self {
            origin: Vec3S::new_zero(),
            direction: Vec3S::new(f32x16::splat(1.0), f32x16::splat(1.0), f32x16::splat(1.0)),
        }
    }
    pub fn origin(&self) -> PointS {
        self.origin
    }
    pub fn direction(&self) -> Vec3S {
        self.direction
    }
    pub fn at(&self, t: f32x16) -> PointS {
        self.origin + (self.direction * t)
    }
}
