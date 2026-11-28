use rand::rand_core::utils::Word;
use wide::{f32x16, u32x16};

use crate::color::{Color, ColorS, write_color};
//use crate::hittable::{HitRecord, Hittable, HittableList};
use crate::Scene;
use crate::interval::{Interval, IntervalS};
use crate::ray::{Ray, RayS};
use crate::utility::{INF, degrees_to_radians, random_double};
use crate::vec as myvec;
use crate::vec_simd::{PointS, Vec3S};
use myvec::{Point, Vec3};
use rayon::{collections, prelude::*};
use std::collections::HashSet;
use std::io::{self, Write};
use std::vec::Vec;

pub struct CameraConfig {
    pub aspect_ratio: f32,
    pub image_width: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
    pub vfov: f32,
    pub lookfrom: Point,
    pub lookat: Point,
    pub vup: Vec3,
    pub defocus_angle: f32,
    pub focus_dist: f32,
}

pub struct Camera {
    aspect_ratio: f32,
    image_width: u32,
    image_height: u32,
    center: Point,
    pixel00_loc: Point,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    samples_per_pixel: u32,
    pixel_samples_scale: f32,
    max_depth: u32,
    vfov: f32,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    defocus_angle: f32,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn new(config: CameraConfig) -> Self {
        let image_height = (config.image_width as f32 / config.aspect_ratio) as u32;

        // determine viewport dimensions
        let theta = degrees_to_radians(config.vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * config.focus_dist;
        let viewport_width = viewport_height * (config.image_width as f32 / image_height as f32);

        // Calculate the u, v, w unit basis vectors for the camera coordinate frame
        let w = (config.lookfrom - config.lookat).unit();
        let u = (config.vup.cross(w)).unit();
        let v = w.cross(u).unit();

        // calculate the vectors across the horizontal and down the vertical viewport edges
        let viewport_u = u * viewport_width;
        let viewport_v = v * viewport_height * -1.0;

        // calculate the horizontal and vertical delta vectors from pixel to pixel
        let pixel_delta_u = viewport_u / config.image_width as f32;
        let pixel_delta_v = viewport_v / image_height as f32;

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
            pixel_samples_scale: 1.0 / config.samples_per_pixel as f32,
            max_depth: config.max_depth,
            vfov: config.vfov,
            u,
            v,
            w,
            defocus_angle: config.defocus_angle,
            defocus_disk_u: u * defocus_radius,
            defocus_disk_v: v * defocus_radius,
        }
    }

    fn ray_color(ray: &Ray, depth: u32, scene: &Scene) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }
        if let Some(hit) = scene.hit(ray, &Interval::new(0.001, INF)) {
            let mut scattered = Ray::default();
            let mut attenuation = Color::new_zero();
            let did_scattered =
                scene
                    .get_mat(hit.mat_idx)
                    .scatter(&ray, &hit, &mut attenuation, &mut scattered);
            if did_scattered {
                return attenuation * Self::ray_color(&scattered, depth - 1, scene);
            }
            return Color::new_zero();
        }

        let unit_direction = ray.direction().unit();
        let a = (unit_direction.y + 1.0) * 0.5;
        Color::new(1.0, 1.0, 1.0) * (1.0 - a) + Color::new(0.5, 0.7, 1.0) * a
    }

    fn ray_color_simd(ray: &RayS, depth: f32x16, active: f32x16, scene: &Scene) -> ColorS {
        let active = active & depth.simd_gt(f32x16::ZERO);
        if active == f32x16::ZERO {
            return ColorS::new_zero();
        }
       
        let hit = scene.hit_simd(&ray, IntervalS::new(f32x16::splat(0.001), f32x16::splat(INF)));
        if hit.hit_mask != f32x16::ZERO {
            let mut scattered = RayS::default();
            let mut attenuation = ColorS::new_zero();

            let hit_mats_unique = HashSet::from(hit.mat_idx.to_array());
            let mut did_scattered = f32x16::ZERO;
            for mat in hit_mats_unique {
                let cur_mat = hit.mat_idx.simd_eq(u32x16::splat(mat)).select(
                    f32x16::ONE,
                    f32x16::ZERO
                );
                did_scattered |= scene.get_mat(mat as usize).scatter_simd(&ray, &hit, &mut attenuation, &mut scattered) & cur_mat;
            }
            if did_scattered != f32x16::ZERO {
                return attenuation * Self::ray_color_simd(&scattered, depth - f32x16::ONE, active, scene);
            }
            return ColorS::new_zero();
        }

        let unit_direction = ray.direction().unit();
        let a = (unit_direction.y + f32x16::splat(1.0)) * 0.5;
        ColorS::new(f32x16::ONE, f32x16::ONE, f32x16::ONE) * (f32x16::ONE - a) + ColorS::new(f32x16::splat(0.5), f32x16::splat(0.7), f32x16::splat(1.0)) * a
    }

    fn get_ray(&self, i: u32, j: u32) -> Ray {
        // Construct a camera ray originating from the origin and directed at
        // randomly sampled point around the pixel location i, j
        let offset = Self::sample_square();
        let pixel_sample = self.pixel00_loc
            + (self.pixel_delta_u * (i as f32 + offset.x))
            + (self.pixel_delta_v * (j as f32 + offset.y));
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;
        Ray::new(ray_origin, ray_direction)
    }

    fn get_ray_simd(&self, i: f32x16, j: f32x16) -> RayS {
        let offset = Self::sample_square_simd();
        let pixel_sample = Vec3S::from_vec(self.pixel00_loc)
            + Vec3S::from_vec(self.pixel_delta_u) * (i + offset.x)
            + Vec3S::from_vec(self.pixel_delta_v) * (j + offset.y);
        let ray_origin = f32x16::splat(self.defocus_angle).simd_le(f32x16::ZERO).select(
            PointS::from_vec(self.center),
            self.defocus_disk_sample_simd()
        );
        let ray_direction = pixel_sample - ray_origin;
        RayS::new(ray_origin, ray_direction)
    }

    fn defocus_disk_sample(&self) -> Point {
        let p = Vec3::random_in_unit_disk();
        self.center + (self.defocus_disk_u * p.x) + (self.defocus_disk_v * p.y)
    }

    fn defocus_disk_sample_simd(&self) -> PointS {
        let p = Vec3S::random_in_unit_disk();
        PointS::from_vec(self.center) + Vec3S::from_vec(self.defocus_disk_u) * p.x + Vec3S::from_vec(self.defocus_disk_v) * p.y
    }

    fn sample_square() -> Vec3 {
        Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
    }

    fn sample_square_simd() -> Vec3S {
        Vec3S::new(f32x16::splat(random_double() - 0.5), f32x16::splat(random_double() - 0.5), f32x16::ZERO)
    }

    // single threaded
    pub fn render_slow(&self, scene: &Scene) {
        println!("P3\n{} {}\n255", self.image_width, self.image_height);
        for j in 0..self.image_height {
            eprint!("\rLines Remaining: {}", self.image_height - j);
            io::stderr().flush().unwrap();
            for i in 0..self.image_width {
                let mut pixel_color = Color::new_zero();
                for sample in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color += Self::ray_color(&r, self.max_depth, scene);
                }
                write_color(&(pixel_color * self.pixel_samples_scale));
            }
        }
    }

    // multithreaded
    pub fn render_fast(&self, scene: &Scene) {
        println!("P3\n{} {}\n255", self.image_width, self.image_height);
        let mut pixels_out: Vec<Color> =
            vec![Color::new_zero(); (self.image_width * self.image_height) as usize];
        pixels_out
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, item)| {
                let j = idx as u32 / self.image_width;
                let i = idx as u32 % self.image_width;

                let mut pixel_color = Color::new_zero();
                for _ in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color += Self::ray_color(&r, self.max_depth, scene);
                }
                *item = pixel_color * self.pixel_samples_scale;
            });
        for col in pixels_out {
            write_color(&col);
        }
    }

    // multithreaded with simd
    pub fn render_faster(&self, scene: &Scene) {
        println!("P3\n{} {}\n255", self.image_width, self.image_height);
        let mut pixels_out: Vec<Color> =
            vec![Color::new_zero(); (self.image_width * self.image_height) as usize];
        pixels_out.par_chunks_mut(16)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                let idx = chunk_idx * 16;
                let j = idx / self.image_width as usize;
                
                let mut js: [f32; 16] = [0.0; 16];
                let mut is: [f32; 16] = [0.0; 16];
                let mut cur = 0;
                if ((idx + 16) / self.image_width as usize) != j {
                    let stop = self.image_width - (idx as u32 % self.image_width);
                    for i in 0..stop {
                        js[cur] = j as f32;
                        is[cur] = ((idx as u32 + i) % self.image_width) as f32;
                        cur += 1;
                    }
                    for i in stop..16 {
                        js[cur] = (j+1) as f32;
                        is[cur] = ((idx as u32 + i) % self.image_width) as f32;
                        cur += 1;
                    }
                } else {
                    let mut cur = 0;
                    for i in 0..16 {
                        js[cur] = j as f32;
                        is[cur] = ((idx as u32 + i) % self.image_width) as f32;
                        cur += 1;
                    }
                }
                
                let mut pixel_color = ColorS::new_zero();
                for _ in 0..self.samples_per_pixel {
                    let r = self.get_ray_simd(f32x16::new(is), f32x16::new(js));
                    pixel_color += Self::ray_color_simd(&r, f32x16::splat(self.max_depth as f32), f32x16::ONE, scene);
                }
                let colors = pixel_color.to_arr();
                for (local_idx, item) in chunk.iter_mut().enumerate() {
                    *item = colors[local_idx];
                }
            }); 
        for col in pixels_out {
            write_color(&col);
        }
    }
}
