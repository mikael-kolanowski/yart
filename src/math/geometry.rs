use std::sync::Arc;

use super::interval::Interval;
use super::ray::Ray;
use super::vector::{Point3, Vec3};
use crate::Material;

pub struct HitInfo {
    pub location: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub material: Arc<dyn Material>,
}

pub trait Hittable {
    fn check_intersection(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo>;
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Arc<dyn Material>,
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
            material: self.material.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::rendering::material::DummyMaterial;

    use super::*;

    fn unit_sphere(center: Vec3) -> Sphere {
        Sphere {
            center,
            radius: 1.0,
            material: Arc::new(DummyMaterial),
        }
    }

    #[test]
    fn sphere_direct_hit() {
        let sphere = unit_sphere(Vec3::new(0.0, 0.0, -5.0));

        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

        let hit = sphere
            .check_intersection(&ray, Interval::new(0.0, 1000.0))
            .unwrap();

        assert!((hit.t - 4.0).abs() < 1e-6);
    }

    #[test]
    fn sphere_miss() {
        let sphere = unit_sphere(Vec3::new(0.0, 0.0, -5.0));
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0));

        let hit = sphere.check_intersection(&ray, Interval::new(0.0, 1000.0));

        assert!(hit.is_none());
    }

    #[test]
    fn sphere_tangent_hit() {
        let sphere = unit_sphere(Vec3::new(0.0, 1.0, -5.0));
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

        let hit = sphere.check_intersection(&ray, Interval::new(0.001, 1000.0));
        assert!(hit.is_some());

        let rec = hit.unwrap();

        assert!((rec.t - 5.0).abs() < 1e-6);
    }

    #[test]
    fn ray_origin_inside_sphere() {
        let sphere = unit_sphere(Vec3::ZERO);
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));

        let hit = sphere.check_intersection(&ray, Interval::new(0.001, 1000.0));
        assert!(hit.is_some());

        let rec = hit.unwrap();

        assert!(rec.t > 0.0);
    }

    #[test]
    fn sphere_behind_ray() {
        let sphere = unit_sphere(Vec3::new(0.0, 0.0, 5.0));
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

        let hit = sphere.check_intersection(&ray, Interval::new(0.001, 1000.0));
        assert!(hit.is_none());
    }

    #[test]
    fn sphere_interval_rejection() {
        let sphere = unit_sphere(Vec3::new(0.0, 0.0, -5.0));

        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

        // Reject valid hit by shrinking interval
        let hit = sphere.check_intersection(&ray, Interval::new(0.001, 3.0));

        assert!(hit.is_none());
    }

    #[test]
    fn hit_center_front_face() {
        let sphere = unit_sphere(Vec3::ZERO);

        let ray = Ray::new(Vec3::new(0.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0));

        let hit = sphere
            .check_intersection(&ray, Interval::new(0.001, f64::INFINITY))
            .expect("Ray should hit sphere");

        // Hit point should be at z = -1
        assert!((hit.location - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-6);

        // Normal should point straight back toward camera
        assert!((hit.normal - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-6);

        // Normal must be unit length
        assert!((hit.normal.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn hit_offset_point() {
        let sphere = unit_sphere(Vec3::ZERO);

        let ray = Ray::new(
            Vec3::new(0.0, 0.0, -3.0),
            Vec3::new(0.0, 0.2, 1.0).normalized(),
        );

        let hit = sphere
            .check_intersection(&ray, Interval::new(0.001, f64::INFINITY))
            .expect("Ray should hit sphere");

        // Normal must match radial direction
        let expected = (hit.location - Vec3::ZERO).normalized();

        assert!((hit.normal - expected).length() < 1e-6);
    }
}
