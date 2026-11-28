use wide::f32x16;

use crate::utility::INF;

pub struct Interval {
    pub min: f32,
    pub max: f32,
}

pub struct IntervalS {
    pub min: f32x16,
    pub max: f32x16,
}

impl Interval {
    pub fn default() -> Self {
        Self {
            min: -INF,
            max: INF,
        }
    }

    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn size(&self) -> f32 {
        self.max - self.min
    }

    pub fn contains(&self, x: f32) -> bool {
        self.min <= x && x <= self.max
    }

    pub fn surrounds(&self, x: f32) -> bool {
        self.min < x && x < self.max
    }

    pub fn clamp(&self, x: f32) -> f32 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }
}

impl IntervalS {
    pub fn default() -> Self {
        Self {
            min: f32x16::splat(-INF),
            max: f32x16::splat(INF),
        }
    }
    pub fn new(min: f32x16, max: f32x16) -> Self {
        Self { min, max }
    }
    pub fn size(&self) -> f32x16 {
        self.max - self.min
    }
    pub fn contains(&self, x: f32x16) -> f32x16 {
        self.min.simd_le(x) & self.max.simd_ge(x)
    }
    pub fn surrounds(&self, x: f32x16) -> f32x16 {
        self.min.simd_lt(x) & self.max.simd_gt(x)
    }
    pub fn clamp(&self, x: f32x16) -> f32x16 {
        let mut ret = x.simd_lt(self.min).select(
            self.min,
            x,
        );
        ret = ret.simd_gt(self.max).select(
            self.max,
            ret,
        );
        ret
    }
}
