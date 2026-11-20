use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec::{Point, Vec3};

use std::sync::Arc;

#[derive(Clone)]
pub struct HitRecord<'a> {
    pub p: Point,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub mat: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    pub fn new(p: Point, normal: Vec3, t: f64, mat: &'a dyn Material) -> Self {
        Self {
            p,
            normal,
            t,
            front_face: true,
            mat,
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vec3) {
        // Sets the hit record normal vector
        // NOTE: the parameter outward_normal is assumed to have unit length

        self.front_face = ray.direction().dot(*outward_normal) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        } else {
            *outward_normal * -1.0
        };
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<HitRecord> {
        None
    }
}

pub struct HittableList {
    objects: Vec<Arc<dyn Hittable + Send + Sync>>,
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<HitRecord> {
        let mut closest_so_far = ray_t.max;
        let mut rec: Option<HitRecord> = None;

        for object in &self.objects {
            let temp_rec = object.hit(ray, &Interval::new(ray_t.min, closest_so_far));
            if let Some(hit) = temp_rec {
                closest_so_far = hit.t;
                rec = Some(hit);
            }
        }

        rec
    }
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::<Arc<dyn Hittable + Send + Sync>>::new(),
        }
    }
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, object: Arc<dyn Hittable + Send + Sync>) {
        self.objects.push(object);
    }
}
