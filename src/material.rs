use crate::color::Color;
use crate::ray::Ray;
use crate::scene::HitRecord;
use crate::utility::random_double;
use crate::vec::Vec3;

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
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
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
}

pub struct Dialectric {
    refraction_index: f64,
}

impl Dialectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }

    fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
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
}
