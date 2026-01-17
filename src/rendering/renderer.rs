use crate::World;
use crate::color::Color;
use crate::progressbar::ProgressBar;
use rand::Rng;
use std::time::Instant;

use crate::math::{Lerp, Ray, Vec3, geometry::Hittable, interval::Interval};

use super::camera::Camera;
use super::sampler::RandomSampler;

pub struct Renderer {
    pub samples_per_pixel: u32,
    pub max_bounces: u32,
}

impl Renderer {
    pub fn new(samples_per_pixel: u32, max_bounces: u32) -> Self {
        Self {
            samples_per_pixel,
            max_bounces,
        }
    }

    fn ray_color(
        &self,
        ray: Ray,
        max_bounces: u32,
        world: &World,
        sampler: &mut RandomSampler<impl Rng>,
    ) -> Color {
        if max_bounces <= 0 {
            return Color::BLACK;
        }
        if let Some(hit_info) = world.check_intersection(&ray, Interval::new(0.001, f64::INFINITY))
        {
            let normal = hit_info.normal.normalized();
            // let bounce_direction = sampler.random_unit_vector_on_hemisphere(normal);
            let bounce_direction = hit_info.normal + sampler.random_unit_vector();
            return self.ray_color(
                Ray::new(hit_info.location, bounce_direction),
                max_bounces - 1,
                world,
                sampler,
            ) * 0.5;
        }

        let t = 0.5 * (ray.direction.normalized().y + 1.0);
        Color::lerp(Color::WHITE, Color::new(0.5, 0.7, 1.0), t)
    }

    pub fn render(&self, world: &World, camera: &Camera, sampler: &mut RandomSampler<impl Rng>) {
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
                    pixel_color =
                        pixel_color + self.ray_color(ray, self.max_bounces, world, sampler);
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
