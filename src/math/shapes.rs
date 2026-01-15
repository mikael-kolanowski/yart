use super::ray::Ray;
use super::vector::Vec3;

pub struct HitInfo {
    pub location: Vec3,
    pub normal: Vec3,
    pub t: f64,
}

pub trait Hittable {
    fn check_intersection(&self, ray: &Ray) -> Option<HitInfo>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere {
    pub origin: Vec3,
    pub radius: f64,
}

impl Hittable for Sphere {
    fn check_intersection(&self, ray: &Ray) -> Option<HitInfo> {
        Option::None
    }
}
