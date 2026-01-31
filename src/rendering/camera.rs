use super::super::math::*;

pub struct Camera {
    // Output image width in pixels
    pub image_width: u32,
    pub image_height: u32,
    pub center: Point3,
    pub pixel_upper_left: Point3,
    pub pixel_delta_u: Vec3,
    pub pixel_delta_v: Vec3,
}

impl Camera {
    pub fn new(
        aspect_ratio: f64,
        image_width: u32,
        fov: u32,
        look_from: Vec3,
        look_at: Vec3,
    ) -> Self {
        let image_height = {
            let w = (image_width as f64 / aspect_ratio) as u32;
            if w < 1 { 1 } else { w }
        };

        let view_up = Vec3::new(0.0, 1.0, 0.0);
        let center = look_from;

        // Calculate viewport dimensions
        let focal_length = (look_from - look_at).length();
        let theta = (fov as f64).to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focal_length;
        let viewport_width = viewport_height * (image_width as f64) / (image_height as f64);
        let camera_center = Point3::ZERO;

        // Calculate the basis for the camera's coordinate frame
        let w = (look_from - look_at).normalized();
        let u = view_up.cross(w);
        let v = w.cross(u);

        // Calculate the vectors across the horizontal and down the vertical viewport edges
        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        // Calculate the horizontal and vertical delta vectors from pixel to pixel
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let viewport_upper_left =
            camera_center - (focal_length * w) - viewport_u / 2.0 - viewport_v / 2.0;

        let pixel_upper_left = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            image_width,
            image_height,
            center,
            pixel_upper_left,
            pixel_delta_u,
            pixel_delta_v,
        }
    }

    pub fn get_ray(&self, i: i32, j: i32, offset: Vec3) -> Ray {
        let pixel_sample = self.pixel_upper_left
            + ((i as f64 + offset.x) * self.pixel_delta_u)
            + ((j as f64 + offset.y) * self.pixel_delta_v);

        let ray_origin = self.center;
        let ray_direction = pixel_sample - ray_origin;
        Ray::new(ray_origin, ray_direction)
    }
}
