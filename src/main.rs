mod camera;
mod color;
//mod hittable;
mod interval;
mod material;
mod ray;
mod scene;
mod sphere;
mod utility;
mod vec;

use camera::Camera;
//use hittable::*;
use sphere::Sphere;

use crate::{
    camera::CameraConfig,
    color::Color,
    material::{Dialectric, Lambertian, MaterialType, Metal},
    scene::Scene,
    utility::random_double,
    vec::{Point, Vec3},
};

use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() {
    let mut scene = Scene::new();

    let ground_mat = Lambertian::new(Color::new(0.5, 0.5, 0.5));
    let ground_mat_idx = scene.register_material(MaterialType::Lambertian(ground_mat));
    scene.add_object(Sphere::new(
        Point::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_mat_idx,
    ));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_double();
            let center = Point::new(
                a as f64 + 0.9 * random_double(),
                0.2,
                b as f64 + 0.9 * random_double(),
            );

            if (center - Point::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Vec3::random() * Vec3::random();
                    let mat = Lambertian::new(albedo);
                    let mat_idx = scene.register_material(MaterialType::Lambertian(mat));
                    scene.add_object(Sphere::new(center, 0.2, mat_idx));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Vec3::random_range(0.5, 1.0);
                    let fuzz = random_double();
                    let mat = Metal::new(albedo, fuzz);
                    let mat_idx = scene.register_material(MaterialType::Metal((mat)));
                    scene.add_object(Sphere::new(center, 0.2, mat_idx));
                } else {
                    // glass
                    let mat = Dialectric::new(1.5);
                    let mat_idx = scene.register_material(MaterialType::Dielectric(mat));
                    scene.add_object(Sphere::new(center, 0.2, mat_idx));
                }
            }
        }
    }

    let mat1 = Dialectric::new(1.5);
    let mat1_idx = scene.register_material(MaterialType::Dielectric(mat1));
    scene.add_object(Sphere::new(Point::new(0.0, 1.0, 0.0), 1.0, mat1_idx));

    let mat2 = Lambertian::new(Color::new(0.4, 0.2, 0.1));
    let mat2_idx = scene.register_material(MaterialType::Lambertian(mat2));
    scene.add_object(Sphere::new(Point::new(-4.0, 1.0, 0.0), 1.0, mat2_idx));

    let mat3 = Metal::new(Color::new(0.7, 0.6, 0.5), 0.0);
    let mat3_idx = scene.register_material(MaterialType::Metal(mat3));
    scene.add_object(Sphere::new(Point::new(4.0, 1.0, 0.0), 1.0, mat3_idx));

    let config = CameraConfig {
        aspect_ratio: 16.0 / 9.0,
        image_width: 1200,
        samples_per_pixel: 500,
        max_depth: 50,
        vfov: 20.0,
        lookfrom: Point::new(13.0, 2.0, 3.0),
        lookat: Point::new(0.0, 0.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),
        defocus_angle: 0.6,
        focus_dist: 10.0,
    };
    let cam = Camera::new(config);

    let now = Instant::now();
    cam.render_fast(&scene);
    eprint!("Time taken to render: {} seconds", now.elapsed().as_secs());
}
