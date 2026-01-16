mod color;
mod math;
mod progressbar;

use std::time::Instant;

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

fn main() {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = {
        let w = (image_width as f64 / aspect_ratio) as i32;
        if w < 1 { 1 } else { w }
    };

    eprintln!("Output image dimensions: {}x{}", image_width, image_height);

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

    // Camera
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

    // Render
    println!("P3");
    println!("{} {}", image_width, image_height);
    println!("255"); // Max color component

    let mut progress_bar = ProgressBar::new(image_height as usize);
    let rendering_started = Instant::now();
    for j in 0..image_height {
        for i in 0..image_width {
            let pixel_center =
                pixel_upper_left + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;
            let ray = Ray::new(camera_center, ray_direction);

            let pixel_color = ray_color(&ray, &world);
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
