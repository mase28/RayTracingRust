use wide::{f32x16, u32x16};

use crate::interval::{Interval, IntervalS};
use crate::material::Material;
use crate::ray::{Ray, RayS};
use crate::scene::{HitRecord, HitRecordSimd};
use crate::vec::{Point, Vec3};
use crate::vec_simd::{PointS, Vec3S};

use std::sync::Arc;

pub struct Sphere {
    center: Point,
    radius: f32,
    mat_idx: usize,
    center_simd: PointS,
}

impl Sphere {
    pub fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<HitRecord> {
        let oc = self.center - ray.origin();
        let a = ray.direction().dot(*ray.direction());
        let h = ray.direction().dot(oc);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        // Find the nearest root that lies in the acceptable range
        let mut root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let p = ray.at(root);
        let outward_normal = (p - self.center) / self.radius;
        let mut rec = HitRecord::new(p, Vec3::new_zero(), root, self.mat_idx);
        rec.set_face_normal(ray, &outward_normal);

        Some(rec)
    }
    pub fn hit_simd(&self, ray: &RayS, ray_t: IntervalS) -> HitRecordSimd {
        let oc = self.center_simd - ray.origin();
        let a = ray.direction().dot(ray.direction());
        let h = ray.direction().dot(oc);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = h * h - a * c;

        let mut hit_mask = discriminant.simd_gt(f32x16::ZERO);

        //let sqrtd = discriminant.sqrt();
        // Find the nearest root that lies in the acceptable range
        //let mut root = (h - sqrtd) / a;
        //if !ray_t.surrounds(root) {
        //    root = (h + sqrtd) / a;
        //    if !ray_t.surrounds(root) {
        //        return None;
        //    }
        //}
        let sqrtd = discriminant.sqrt();
        let mut root: f32x16 = (h - sqrtd) / a;
        root = ray_t.surrounds(root).select(
            root,
            (h + sqrtd) / a,
        );
        hit_mask = hit_mask & ray_t.surrounds(root);

        let p = ray.at(root);
        let outward_normal = (p - self.center_simd) / f32x16::splat(self.radius);
        let mut rec = HitRecordSimd::new(p, Vec3S::new_zero(), root, u32x16::splat(self.mat_idx as u32), hit_mask);
        rec.set_face_normal(ray, &outward_normal);

        rec
    }
}

impl Sphere {
    pub fn new(center: Point, radius: f32, mat_idx: usize) -> Self {
        Self {
            center,
            radius,
            mat_idx,
            center_simd: PointS::from_vec(center),
        }
    }
}
