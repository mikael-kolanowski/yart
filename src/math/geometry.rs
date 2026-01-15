use super::ray::Ray;
use super::vector::{Point3, Vec3};

pub struct HitInfo {
    pub location: Point3,
    pub normal: Vec3,
    pub t: f64,
}

pub trait Hittable {
    fn check_intersection(&self, ray: &Ray) -> Option<HitInfo>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
}

impl Hittable for Sphere {
    fn check_intersection(&self, ray: &Ray) -> Option<HitInfo> {
        // let oc = self.center - ray.origin;
        // let a = ray.direction.dot(ray.direction);
        // let b = -2.0 * ray.direction.dot(oc);
        // let c = oc.dot(oc) - self.radius * self.radius;
        // let discriminant = b * b - 4.0 * a * c;
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius*self.radius;
        let discriminant = h*h - a*c;

        if discriminant < 0.0 {
            None
        } else {
            let t = (h - discriminant.sqrt()) / a;
            let normal = (ray.at(t) - self.center).normalized();
            Some(HitInfo {
                t: t,
                location: ray.at(t),
                normal: normal,
            })
        }
    }
}
