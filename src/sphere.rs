use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray;
use crate::scene::HitRecord;
use crate::vec::{Point, Vec3};

use std::sync::Arc;

pub struct Sphere {
    center: Point,
    radius: f64,
    mat_idx: usize,
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
        let mut rec = HitRecord::new(p, Vec3::random(), root, self.mat_idx);
        rec.set_face_normal(ray, &outward_normal);

        Some(rec)
    }
}

impl Sphere {
    pub fn new(center: Point, radius: f64, mat_idx: usize) -> Self {
        Self {
            center,
            radius,
            mat_idx,
        }
    }
}
