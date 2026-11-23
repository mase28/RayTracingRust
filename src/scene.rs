//use crate::hittable::HittableList;
use crate::interval::Interval;
use crate::material::MaterialType;
use crate::ray::Ray;
use crate::sphere::Sphere;
use crate::vec::{Point, Vec3};

use std::vec::Vec;

pub struct HitRecord {
    pub p: Point,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub mat_idx: usize,
}

impl HitRecord {
    pub fn new(p: Point, normal: Vec3, t: f64, mat_idx: usize) -> Self {
        Self {
            p,
            normal,
            t,
            front_face: true,
            mat_idx,
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

pub struct Scene {
    objects: Vec<Sphere>,
    materials: Vec<MaterialType>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::<Sphere>::new(),
            materials: Vec::<MaterialType>::new(),
        }
    }

    pub fn register_material(&mut self, mat: MaterialType) -> usize {
        self.materials.push(mat);
        self.materials.len() - 1
    }

    pub fn add_object(&mut self, sphere: Sphere) {
        self.objects.push(sphere);
    }

    pub fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<HitRecord> {
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

    pub fn get_mat(&self, mat_idx: usize) -> &MaterialType {
        &self.materials[mat_idx]
    }
}
