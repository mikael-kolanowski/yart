use super::ray::Ray;
use super::vector::{Vec3, Point3};

pub struct HitInfo {
    pub location: Point3,
    pub normal: Vec3,
    pub t: f64,
}

pub trait Hittable {
    fn check_intersection(&self, ray: &Ray) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
}

impl Hittable for Sphere {
    fn check_intersection(&self, ray: &Ray) -> bool {
        let oc = self.center - ray.origin;
        let a = ray.direction.dot(ray.direction);
        let b = -2.0 * ray.direction.dot(oc);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;
        return discriminant >= 0.0;
    }
}
