use wide::bytemuck::Zeroable;
use wide::{Select, f32x16, u32x16};

use crate::color::Color;
//use crate::hittable::HittableList;
use crate::interval::{Interval, IntervalS};
use crate::material::{Lambertian, MaterialType};
use crate::ray::{Ray, RayS};
use crate::sphere::Sphere;
use crate::utility::INF;
use crate::vec::{Point, Vec3};
use crate::vec_simd::{PointS, Vec3S};

use std::vec::Vec;

pub struct HitRecord {
    pub p: Point,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool,
    pub mat_idx: usize,
}

pub struct HitRecordSimd {
    pub p: PointS,
    pub normal: Vec3S,
    pub t: f32x16,
    pub front_face: f32x16,
    pub mat_idx: u32x16,
    pub hit_mask: f32x16,
}

impl HitRecord {
    pub fn new(p: Point, normal: Vec3, t: f32, mat_idx: usize) -> Self {
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

impl HitRecordSimd {
    pub fn new(p: PointS, normal: Vec3S, t: f32x16, mat_idx: u32x16, hit_mask: f32x16) -> Self {
        Self {
            p,
            normal,
            t,
            front_face: f32x16::splat(1.0),
            mat_idx,
            hit_mask,
        }
    }
    pub fn set_face_normal(&mut self, ray: &RayS, outward_normal: &Vec3S) {
        self.front_face = ray.direction().dot(*outward_normal).is_sign_negative();
        self.normal = self.front_face.select(
            *outward_normal,
            *outward_normal * f32x16::splat(-1.0),
        );
    }

    pub fn miss(t: f32x16) -> Self {
        Self {
            p: PointS::new_zero(),
            normal: Vec3S::new_zero(),
            t,
            front_face: f32x16::splat(1.0),
            mat_idx: u32x16::ZERO,
            hit_mask: f32x16::ZERO,
        }
    }

    // assumes that other has a closer interval hit
    pub fn merge_closer(&mut self, other: &Self) {
        let closer: f32x16 = other.t.simd_lt(self.t);
        let mask: f32x16 = other.hit_mask & closer;
        self.p = mask.select(
            other.p, 
            self.p
        );
        self.normal = mask.select(
            other.normal, 
            self.normal
        );
        self.t = mask.select(
            other.t,
            self.t
        );
        self.front_face = mask.select(
            other.front_face, 
            self.front_face
        );
        self.mat_idx = mask.select(
            other.mat_idx, 
            self.mat_idx
        );
        self.hit_mask = self.hit_mask | other.hit_mask;
    }
}

pub struct Scene {
    objects: Vec<Sphere>,
    materials: Vec<MaterialType>,
}

impl Scene {
    pub fn new() -> Self {
        let mut base_mat = Vec::<MaterialType>::new();
        base_mat.push(MaterialType::Lambertian(Lambertian::new(Color::new_zero())));
        Self {
            objects: Vec::<Sphere>::new(),
            materials: base_mat,
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

    pub fn hit_simd(&self, ray: &RayS, ray_t: IntervalS) -> HitRecordSimd {
        let mut rec = HitRecordSimd::miss(f32x16::splat(INF));

        for object in &self.objects {
            let hit = object.hit_simd(ray, IntervalS::new(ray_t.min, ray_t.max));
            if hit.hit_mask == f32x16::ZERO {
                continue;
            }
            // below will only merge rays with closer hits otherwise stays the same
            rec.merge_closer(&hit); 
        }
        rec
    }

    pub fn get_mat(&self, mat_idx: usize) -> &MaterialType {
        &self.materials[mat_idx]
    }
}
