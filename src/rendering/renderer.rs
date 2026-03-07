use log::info;

use crate::World;
use crate::color::Color;
use crate::image::Image;
use crate::progressbar::ProgressBar;
use std::time::Instant;

use crate::math::{Ray, geometry::Hittable, interval::Interval};

use super::camera::Camera;
use super::sampler::Sampler;

pub struct Renderer {
    samples_per_pixel: u32,
    max_bounces: u32,
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
        sampler: &mut dyn Sampler,
    ) -> Color {
        if max_bounces == 0 {
            return Color::BLACK;
        }
        if let Some(hit_info) = world.check_intersection(&ray, Interval::new(0.001, f64::INFINITY))
        {
            if let Some((attenuation, scattered)) =
                hit_info.material.scatter(ray, &hit_info, sampler)
            {
                return attenuation * self.ray_color(scattered, max_bounces - 1, world, sampler);
            } else {
                return hit_info.material.emitted(&hit_info);
            }
        }

        world.skybox.color(ray)
    }

    pub fn render(
        &self,
        world: &World,
        camera: &Camera,
        sampler: &mut dyn Sampler,
        show_progress: bool,
    ) -> Image {
        info!(
            "output image dimensions: {}x{}",
            camera.image_width, camera.image_height
        );

        let mut image = Image::new(camera.image_width, camera.image_height);

        let pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        let mut progress_bar = ProgressBar::new("Rendering".to_string(), camera.image_height);
        let rendering_started = Instant::now();
        for j in 0..camera.image_height {
            for i in 0..camera.image_width {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..self.samples_per_pixel {
                    let offset = sampler.in_square();
                    let ray = camera.get_ray(i as i32, j as i32, offset);
                    pixel_color =
                        pixel_color + self.ray_color(ray, self.max_bounces, world, sampler);
                }
                pixel_color = pixel_color * pixel_samples_scale;
                image.add_pixel(pixel_color);
            }
            if show_progress {
                progress_bar.increment();
            }
        }
        let rendering_finished = Instant::now();
        progress_bar.finish();
        info!(
            "image rendered in {} ms",
            (rendering_finished - rendering_started).as_millis()
        );
        image
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::SmallRng};

    use crate::{math::Point3, rendering::sampler::RandomSampler};

    use super::*;
    #[test]
    fn renderer_smoke_test() {
        let rng = SmallRng::seed_from_u64(1337);
        let mut sampler = RandomSampler::new(rng);

        let camera = Camera::new(1.0, 32, 90, Point3::ORIGIN, Point3::new(0.0, -1.0, 0.0));

        let world = World::default();

        let renderer = Renderer::new(1, 1);

        let image = renderer.render(&world, &camera, &mut sampler, false);

        assert_eq!(image.pixels.len(), 32 * 32);
    }
}
