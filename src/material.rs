use std::array;

use wide::f32x16;

use crate::color::{Color, ColorS};
use crate::ray::{Ray, RayS};
use crate::scene::{HitRecord, HitRecordSimd};
use crate::utility::random_double;
use crate::vec::Vec3;
use crate::vec_simd::Vec3S;

pub trait Material: Send + Sync {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        false
    }

    fn scatter_simd(&self, r_in: &RayS, rec: &HitRecordSimd, attenuation: &mut ColorS, scattered: &mut RayS) -> f32x16 {
        f32x16::ZERO
    }
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let mut scatter_direction = rec.normal + Vec3::random_unit();
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }
        *scattered = Ray::new(rec.p, scatter_direction);
        *attenuation = self.albedo;
        true
    }

    fn scatter_simd(&self, r_in: &RayS, rec: &HitRecordSimd, attenuation: &mut ColorS, scattered: &mut RayS) -> f32x16 {
        let mut scatter_direction = rec.normal + Vec3S::random_unit();
        scatter_direction = scatter_direction.near_zero().select(
            rec.normal, 
            scatter_direction
        );
        *scattered = rec.hit_mask.select(
            RayS::new(rec.p, scatter_direction),
            *scattered
        );
        *attenuation = rec.hit_mask.select(
            Vec3S::from_vec(self.albedo),
            *attenuation
        );
        f32x16::ONE
    }
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f32) -> Self {
        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let reflected =
            Vec3::reflect(*r_in.direction(), rec.normal).unit() + (Vec3::random_unit() * self.fuzz);
        *scattered = Ray::new(rec.p, reflected);
        *attenuation = self.albedo;
        scattered.direction().dot(rec.normal) > 0.0
    }

    fn scatter_simd(&self, r_in: &RayS, rec: &HitRecordSimd, attenuation: &mut ColorS, scattered: &mut RayS) -> f32x16 {
        let reflected = Vec3S::reflect(r_in.direction(), rec.normal).unit() + Vec3S::random_unit() * f32x16::splat(self.fuzz);   
        *scattered = rec.hit_mask.select(
            RayS::new(rec.p, reflected),
            *scattered
        );
        *attenuation = rec.hit_mask.select(
            ColorS::from_vec(self.albedo),
            *attenuation
        );
        scattered.direction().dot(rec.normal).simd_gt(f32x16::ZERO)
    }
}

pub struct Dialectric {
    refraction_index: f32,
}

impl Dialectric {
    pub fn new(refraction_index: f32) -> Self {
        Self { refraction_index }
    }

    fn reflectance(cosine: f32, refraction_index: f32) -> f32 {
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
    fn reflectance_simd(cosine: f32x16, refraction_index: f32x16) -> f32x16 {
        let mut r0 = (f32x16::ONE - refraction_index) / (f32x16::ONE + refraction_index);
        r0 = r0 * r0;
        r0 + (f32x16::ONE - r0) * (f32x16::ONE - cosine).powf_simd(f32x16::splat(5.0))
    }
}

impl Material for Dialectric {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        *attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = r_in.direction().unit();
        let cos_theta = (unit_direction.dot(rec.normal * -1.0)).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction: Vec3;

        if cannot_refract || Self::reflectance(cos_theta, ri) > random_double() {
            direction = Vec3::reflect(unit_direction, rec.normal);
        } else {
            direction = Vec3::refract(unit_direction, rec.normal, ri);
        }
        *scattered = Ray::new(rec.p, direction);
        true
    }

    fn scatter_simd(&self, r_in: &RayS, rec: &HitRecordSimd, attenuation: &mut ColorS, scattered: &mut RayS) -> f32x16 {
        *attenuation = rec.hit_mask.select(
            ColorS::new(f32x16::ONE, f32x16::ONE, f32x16::ONE),
            *attenuation
        );
        let ri = rec.front_face.simd_eq(f32x16::ONE).select(
            f32x16::splat(1.0 / self.refraction_index), 
            f32x16::splat(self.refraction_index)
        );

        let unit_direction = r_in.direction().unit();
        let cos_theta = unit_direction.dot(rec.normal * f32x16::splat(-1.0)).min(f32x16::ONE);
        let sin_theta = (f32x16::ONE - cos_theta * cos_theta).sqrt();

        let cannot_refract = (ri * sin_theta).simd_gt(f32x16::ONE);
        let rand_arr: [f32; 16] = array::from_fn(|_| random_double());
        let reflectance = Self::reflectance_simd(cos_theta, ri).simd_gt(f32x16::new(rand_arr));
        
        let direction = (cannot_refract | reflectance).select(
            Vec3S::reflect(unit_direction, rec.normal),
            Vec3S::refract(unit_direction, rec.normal, ri)
        );
        *scattered = rec.hit_mask.select(
            RayS::new(rec.p, direction),
            *scattered
        );
        f32x16::ONE
    }
}

pub enum MaterialType {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dialectric),
}

impl MaterialType {
    pub fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        match self {
            MaterialType::Lambertian(m) => m.scatter(r_in, rec, attenuation, scattered),
            MaterialType::Metal(m) => m.scatter(r_in, rec, attenuation, scattered),
            MaterialType::Dielectric(m) => m.scatter(r_in, rec, attenuation, scattered),
        }
    }
    pub fn scatter_simd(
        &self,
        r_in: &RayS,
        rec: &HitRecordSimd,
        attenuation: &mut ColorS,
        scattered: &mut RayS,
    ) -> f32x16 {
        match self {
            MaterialType::Lambertian(m) => m.scatter_simd(r_in, rec, attenuation, scattered),
            MaterialType::Metal(m) => m.scatter_simd(r_in, rec, attenuation, scattered),
            MaterialType::Dielectric(m) => m.scatter_simd(r_in, rec, attenuation, scattered),
        }
    }
}
