#![allow(dead_code)]

use crate::vec::{Point, Vec3};

pub struct Ray {
    origin: Point,
    direction: Vec3,
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

    pub fn at(&self, t: f64) -> Point {
        self.origin + (self.direction * t)
    }
}
