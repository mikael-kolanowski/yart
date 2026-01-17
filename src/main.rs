mod color;
mod math;
mod progressbar;

use std::time::Instant;

use rand::Rng;

use crate::color::Color;
use crate::math::interval::Interval;
use crate::math::*;
use crate::progressbar::ProgressBar;

struct World {
    objects: Vec<Box<dyn Hittable>>,
}

impl World {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for World {
    fn check_intersection(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        let mut closest = ray_t.max;
        let mut hit_anything = None;

        for obj in &self.objects {
            if let Some(hit) = obj.check_intersection(ray, Interval::new(ray_t.min, closest)) {
                closest = hit.t;
                hit_anything = Some(hit)
            }
        }
        return hit_anything;
    }
}

fn ray_color(ray: &Ray, world: &World) -> Color {
    if let Some(hit_info) = world.check_intersection(ray, Interval::new(0.0, f64::INFINITY)) {
        let normal = hit_info.normal.normalized();
        return Color::from(0.5 * (normal + Vec3::ONES));
    }

    let t = 0.5 * (ray.direction.normalized().y + 1.0);
    Color::lerp(Color::WHITE, Color::new(0.5, 0.7, 1.0), t)
}

struct Camera {
    // Ratio of image width over height
    aspect_ratio: f64,
    // Output image width in pixels
    image_width: usize,
    image_height: usize,
    center: Point3,
    pixel_upper_left: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

impl Camera {
    fn new(aspect_ratio: f64, image_width: usize) -> Self {
        let image_height = {
            let w = (image_width as f64 / aspect_ratio) as usize;
            if w < 1 { 1 } else { w }
        };

        let center = Point3::ZERO;

        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (image_width as f64) / (image_height as f64);
        let camera_center = Point3::ZERO;

        // Calculate the vectors across the horizontal and down the vertical viewport edges
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

        // Calculate the horizontal and vertical delta vectors from pixel to pixel
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let viewport_upper_left =
            camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;

        let pixel_upper_left = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            aspect_ratio,
            image_width,
            image_height,
            center,
            pixel_upper_left,
            pixel_delta_u,
            pixel_delta_v,
        }
    }

    fn get_ray(&self, i: i32, j: i32, offset: Vec3) -> Ray {
        let pixel_sample = self.pixel_upper_left
            + ((i as f64 + offset.x) * self.pixel_delta_u)
            + ((j as f64 + offset.y) * self.pixel_delta_v);

        let ray_origin = self.center;
        let ray_direction = pixel_sample - ray_origin;
        Ray::new(ray_origin, ray_direction)
    }
}

struct Renderer {
    samples_per_pixel: u32,
}

impl Renderer {
    fn new(samples_per_pixel: u32) -> Self {
        Self { samples_per_pixel }
    }

    /// Returns the vector to a random point in the [-0.5, -0.5]-[0.5, 0.5] unit square
    fn sample_square(&self, rng: &mut impl Rng) -> Vec3 {
        let a: f64 = rng.random();
        let b: f64 = rng.random();
        Vec3::new(a - 0.5, b - 0.5, 0.0)
    }

    fn render(&self, world: &World, camera: &Camera, rng: &mut impl Rng) {
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
                    let offset = self.sample_square(rng);
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

fn main() {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;

    // World
    let mut world = World::new();
    world.add(Box::new(geometry::Sphere {
        center: Point3::new(0.0, 0.0, -1.0),
        radius: 0.5,
    }));

    world.add(Box::new(geometry::Sphere {
        center: Point3::new(0.0, -100.5, -1.0),
        radius: 100.0,
    }));

    let mut rng = rand::rng();

    let camera = Camera::new(aspect_ratio, image_width);

    let renderer = Renderer::new(100);

    renderer.render(&world, &camera, &mut rng);
}
