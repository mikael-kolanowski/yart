mod math;

use std::time::Instant;

use crate::math::*;

#[derive(Clone, Copy)]
struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };

    fn lerp(start: Color, end: Color, t: f64) -> Self {
        let va = math::Vec3::new(start.r, start.g, start.b);
        let vb = math::Vec3::new(end.r, end.g, end.b);
        let lerped = Vec3::lerp(va, vb, t);
        Color {
            r: lerped.x,
            g: lerped.y,
            b: lerped.z,
        }
    }

    fn write(&self) {
        let ir = (255.999 * self.r) as i32;
        let ig = (255.999 * self.g) as i32;
        let ib = (255.999 * self.b) as i32;

        println!("{} {} {}", ir, ig, ib);
    }
}

fn ray_color(ray: &Ray) -> Color {
    let sphere = math::shapes::Sphere {
        center: math::Point3::new(0.0, 0.0, -1.0),
        radius: 0.5,
    };

    if sphere.check_intersection(ray) {
        return Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        };
    }

    let t = 0.5 * (ray.direction.normalized().y + 1.0);
    Color::lerp(
        Color::WHITE,
        Color {
            r: 0.5,
            g: 0.7,
            b: 1.0,
        },
        t,
    )
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

    // Camera
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width as f64) / (image_height as f64);
    let camera_center = math::Point3::ZERO;

    // Calculate the vectors across the horizontal and down the vertical viewport edges
    let viewport_u = math::Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = math::Vec3::new(0.0, viewport_height, 0.0);

    // Calculate the horizontal and vertical delta vectors from pixel to pixel
    let pixel_delta_u = viewport_u / image_width as f64;
    let pixel_delta_v = viewport_v / image_height as f64;

    let viewport_upper_left = camera_center
        - math::Vec3::new(0.0, 0.0, focal_length)
        - viewport_u / 2.0
        - viewport_v / 2.0;

    let pixel_upper_left = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    // Render
    println!("P3");
    println!("{} {}", image_width, image_height);
    println!("255"); // Max color component

    let rendering_started = Instant::now();
    for j in 0..image_height {
        eprintln!("Scanline {} / {}", j + 1, image_height);
        for i in 0..image_width {
            let pixel_center =
                pixel_upper_left + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;
            let ray = math::Ray::new(camera_center, ray_direction);

            let pixel_color = ray_color(&ray);
            pixel_color.write();
        }
    }
    let rendering_finished = Instant::now();
    eprintln!("Image rendered in {} ms", (rendering_finished - rendering_started).as_millis());
}
