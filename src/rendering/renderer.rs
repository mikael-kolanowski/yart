use crate::World;
use crate::color::Color;
use crate::progressbar::ProgressBar;
use rand::Rng;
use std::time::Instant;

use crate::math::{Lerp, Ray, Vec3, geometry::Hittable, interval::Interval};

use super::camera::Camera;
use super::sampler::Sampler;

fn ray_color(ray: &Ray, world: &World) -> Color {
    if let Some(hit_info) = world.check_intersection(ray, Interval::new(0.0, f64::INFINITY)) {
        let normal = hit_info.normal.normalized();
        return Color::from(0.5 * (normal + Vec3::ONES));
    }

    let t = 0.5 * (ray.direction.normalized().y + 1.0);
    Color::lerp(Color::WHITE, Color::new(0.5, 0.7, 1.0), t)
}

pub struct Renderer {
    pub samples_per_pixel: u32,
}

impl Renderer {
    pub fn new(samples_per_pixel: u32) -> Self {
        Self { samples_per_pixel }
    }

    pub fn render(&self, world: &World, camera: &Camera, sampler: &mut Sampler<impl Rng>) {
        eprintln!(
            "Output image dimensions: {}x{}",
            camera.image_width, camera.image_height
        );

        let pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        // Render
        println!("P3");
        println!("{} {}", camera.image_width, camera.image_height);
        println!("255"); // Max color component

        // TODO: Consider rectoring out image generation
        let mut progress_bar = ProgressBar::new("Rendering".to_string(), camera.image_height);
        let rendering_started = Instant::now();
        for j in 0..camera.image_height {
            for i in 0..camera.image_width {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..self.samples_per_pixel {
                    let offset = sampler.sample_square();
                    let ray = camera.get_ray(i as i32, j as i32, offset);
                    pixel_color = pixel_color + ray_color(&ray, world);
                }
                pixel_color = pixel_color * pixel_samples_scale;
                pixel_color.write();
            }
            progress_bar.increment();
        }
        let rendering_finished = Instant::now();
        progress_bar.finish();
        eprintln!(
            "Image rendered in {} ms",
            (rendering_finished - rendering_started).as_millis()
        );
    }
}
