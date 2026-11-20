mod camera;
mod color;
mod hittable;
mod interval;
mod material;
mod ray;
mod sphere;
mod utility;
mod vec;

use camera::Camera;
use hittable::*;
use rand::seq::WeightError::InsufficientNonZero;
use sphere::Sphere;

use crate::{
    camera::CameraConfig,
    color::Color,
    material::{Dialectric, Lambertian, Metal},
    utility::random_double,
    vec::{Point, Vec3},
};

use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() {
    let mut world = HittableList::new();

    let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Arc::new(Sphere::new(
        Point::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_mat,
    )));

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
                    let mat = Arc::new(Lambertian::new(albedo));
                    world.add(Arc::new(Sphere::new(center, 0.2, mat)));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Vec3::random_range(0.5, 1.0);
                    let fuzz = random_double();
                    let mat = Arc::new(Metal::new(albedo, fuzz));
                    world.add(Arc::new(Sphere::new(center, 0.2, mat)));
                } else {
                    // glass
                    let mat = Arc::new(Dialectric::new(1.5));
                    world.add(Arc::new(Sphere::new(center, 0.2, mat)));
                }
            }
        }
    }

    let mat1 = Arc::new(Dialectric::new(1.5));
    world.add(Arc::new(Sphere::new(Point::new(0.0, 1.0, 0.0), 1.0, mat1)));

    let mat2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Arc::new(Sphere::new(Point::new(-4.0, 1.0, 0.0), 1.0, mat2)));

    let mat3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Arc::new(Sphere::new(Point::new(4.0, 1.0, 0.0), 1.0, mat3)));

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
    cam.render_fast(&world);
    println!("Time taken to render: {} seconds", now.elapsed().as_secs());
}
