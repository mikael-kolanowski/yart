use super::interval::Interval;
use super::ray::Ray;
use super::vector::{Point3, Vec3};

pub struct HitInfo {
    pub location: Point3,
    pub normal: Vec3,
    pub t: f64,
}

pub trait Hittable {
    fn check_intersection(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
}

impl Hittable for Sphere {
    fn check_intersection(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let mut root = (h - sqrt_d) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrt_d) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let point = ray.at(root);
        let normal = (point - self.center) / self.radius;
        return Some(HitInfo {
            location: point,
            normal: normal,
            t: root,
        });
    }
}
