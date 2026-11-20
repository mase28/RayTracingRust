use rand::rand_core::utils::Word;

use crate::color::{Color, write_color};
use crate::hittable::{HitRecord, Hittable, HittableList};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::utility::{INF, degrees_to_radians, random_double};
use crate::vec as myvec;
use myvec::{Point, Vec3};
use rayon::prelude::*;
use std::array::from_fn;
use std::io::{self, Write};
use std::vec::Vec;

pub struct CameraConfig {
    pub aspect_ratio: f64,
    pub image_width: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
    pub vfov: f64,
    pub lookfrom: Point,
    pub lookat: Point,
    pub vup: Vec3,
    pub defocus_angle: f64,
    pub focus_dist: f64,
}

pub struct Camera {
    aspect_ratio: f64,
    image_width: u32,
    image_height: u32,
    center: Point,
    pixel00_loc: Point,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    samples_per_pixel: u32,
    pixel_samples_scale: f64,
    max_depth: u32,
    vfov: f64,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    defocus_angle: f64,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
    pixels_out: Vec<Color>,
}

impl Camera {
    pub fn new(config: CameraConfig) -> Self {
        let image_height = (config.image_width as f64 / config.aspect_ratio) as u32;

        // determine viewport dimensions
        let theta = degrees_to_radians(config.vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * config.focus_dist;
        let viewport_width = viewport_height * (config.image_width as f64 / image_height as f64);

        // Calculate the u, v, w unit basis vectors for the camera coordinate frame
        let w = (config.lookfrom - config.lookat).unit();
        let u = (config.vup.cross(w)).unit();
        let v = w.cross(u).unit();

        // calculate the vectors across the horizontal and down the vertical viewport edges
        let viewport_u = u * viewport_width;
        let viewport_v = v * viewport_height * -1.0;

        // calculate the horizontal and vertical delta vectors from pixel to pixel
        let pixel_delta_u = viewport_u / config.image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // calculate the location of the upper left pixel
        let center = config.lookfrom;
        let viewport_upper_left =
            center - w * config.focus_dist - viewport_u / 2.0 - viewport_v / 2.0;

        let defocus_radius =
            config.focus_dist * (degrees_to_radians(config.defocus_angle / 2.0)).tan();

        Self {
            aspect_ratio: config.aspect_ratio,
            image_width: config.image_width,
            image_height: if image_height < 1 { 1 } else { image_height },
            center: center,
            pixel00_loc: viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5,
            pixel_delta_u: pixel_delta_u,
            pixel_delta_v: pixel_delta_v,
            samples_per_pixel: config.samples_per_pixel,
            pixel_samples_scale: 1.0 / config.samples_per_pixel as f64,
            max_depth: config.max_depth,
            vfov: config.vfov,
            u,
            v,
            w,
            defocus_angle: config.defocus_angle,
            defocus_disk_u: u * defocus_radius,
            defocus_disk_v: v * defocus_radius,
            pixels_out: Vec::with_capacity((image_height * config.image_width) as usize),
        }
    }

    fn ray_color(ray: &Ray, depth: u32, world: &HittableList) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }
        if let Some(hit) = world.hit(ray, &Interval::new(0.001, INF)) {
            let mut scattered = Ray::default();
            let mut attenuation = Color::new_zero();
            let did_scattered = hit
                .mat
                .scatter(&ray, &hit, &mut attenuation, &mut scattered);
            if did_scattered {
                return attenuation * Self::ray_color(&scattered, depth - 1, &world);
            }
            return Color::new_zero();
        }

        let unit_direction = ray.direction().unit();
        let a = (unit_direction.y + 1.0) * 0.5;
        Color::new(1.0, 1.0, 1.0) * (1.0 - a) + Color::new(0.5, 0.7, 1.0) * a
    }

    fn get_ray(&self, i: u32, j: u32) -> Ray {
        // Construct a camera ray originating from the origin and directed at
        // randomly sampled point around the pixel location i, j
        let offset = Self::sample_square();
        let pixel_sample = self.pixel00_loc
            + (self.pixel_delta_u * (i as f64 + offset.x))
            + (self.pixel_delta_v * (j as f64 + offset.y));
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;
        Ray::new(ray_origin, ray_direction)
    }

    fn defocus_disk_sample(&self) -> Point {
        let p = Vec3::random_in_unit_disk();
        self.center + (self.defocus_disk_u * p.x) + (self.defocus_disk_v * p.y)
    }

    fn sample_square() -> Vec3 {
        Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
    }

    pub fn render_slow(&self, world: &HittableList) {
        println!("P3\n{} {}\n255", self.image_width, self.image_height);
        for j in 0..self.image_height {
            eprint!("\rLines Remaining: {}", self.image_height - j);
            io::stderr().flush().unwrap();
            for i in 0..self.image_width {
                let mut pixel_color = Color::new_zero();
                for sample in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color += Self::ray_color(&r, self.max_depth, world);
                }
                write_color(&(pixel_color * self.pixel_samples_scale));
            }
        }
    }

    pub fn render_fast(&mut self, world: &HittableList) {
        println!("P3\n{} {}\n255", self.image_width, self.image_height);
        self.pixels_out
            .par_iter_mut()
            .enumerate()
            .map(|(idx, item)| {
                let j = idx as u32 / self.image_height;
                let i = idx as u32 - (self.image_height * j);

                let mut pixel_color = Color::new_zero();
                for sample in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color += Self::ray_color(&r, self.max_depth, world);
                }
                *item = pixel_color * self.pixel_samples_scale;
            });
    }
}
