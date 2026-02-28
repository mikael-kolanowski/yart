use std::sync::Arc;

use super::interval::Interval;
use super::ray::Ray;
use super::vector::{Normal3, Point3};
use crate::Material;

pub struct HitInfo {
    pub point: Point3,
    pub normal: Normal3,
    pub t: f64,
    pub front_face: bool,
    pub material: Arc<dyn Material>,
}

pub trait Hittable {
    fn check_intersection(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo>;
}

pub struct Sphere {
    pub center: Point3,
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
        let outward_normal = Normal3((point - self.center) / self.radius);
        let mut normal = outward_normal;

        let front_face = {
            if ray.direction.dot(outward_normal.0) > 0.0 {
                normal = -outward_normal;
                false
            } else {
                true
            }
        };

        return Some(HitInfo {
            point,
            normal,
            t: root,
            front_face,
            material: self.material.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use core::f64;

    use crate::math::Vec3;
    use crate::rendering::material::DummyMaterial;

    use super::*;

    fn unit_sphere(center: Point3) -> Sphere {
        Sphere {
            center,
            radius: 1.0,
            material: Arc::new(DummyMaterial),
        }
    }

    fn sphere(center: Point3, radius: f64) -> Sphere {
        Sphere {
            center,
            radius,
            material: Arc::new(DummyMaterial),
        }
    }

    #[test]
    fn sphere_direct_hit() {
        let sphere = unit_sphere(Point3::new(0.0, 0.0, -5.0));

        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 0.0, -1.0));

        let hit = sphere
            .check_intersection(&ray, Interval::new(0.0, 1000.0))
            .unwrap();

        assert!((hit.t - 4.0).abs() < 1e-6);
    }

    #[test]
    fn sphere_miss() {
        let sphere = unit_sphere(Point3::new(0.0, 0.0, -5.0));
        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 1.0, 0.0));

        let hit = sphere.check_intersection(&ray, Interval::new(0.0, 1000.0));

        assert!(hit.is_none());
    }

    #[test]
    fn sphere_tangent_hit() {
        let sphere = unit_sphere(Point3::new(0.0, 1.0, -5.0));
        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 0.0, -1.0));

        let hit = sphere.check_intersection(&ray, Interval::new(0.001, 1000.0));
        assert!(hit.is_some());

        let rec = hit.unwrap();

        assert!((rec.t - 5.0).abs() < 1e-6);
    }

    #[test]
    fn ray_origin_inside_sphere() {
        let sphere = unit_sphere(Point3::ORIGIN);
        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 0.0, 1.0));

        let hit = sphere.check_intersection(&ray, Interval::new(0.001, 1000.0));
        assert!(hit.is_some());

        let rec = hit.unwrap();

        assert!(rec.t > 0.0);
    }

    #[test]
    fn sphere_behind_ray() {
        let sphere = unit_sphere(Point3::new(0.0, 0.0, 5.0));
        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 0.0, -1.0));

        let hit = sphere.check_intersection(&ray, Interval::new(0.001, 1000.0));
        assert!(hit.is_none());
    }

    #[test]
    fn sphere_interval_rejection() {
        let sphere = unit_sphere(Point3::new(0.0, 0.0, -5.0));

        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 0.0, -1.0));

        // Reject valid hit by shrinking interval
        let hit = sphere.check_intersection(&ray, Interval::new(0.001, 3.0));

        assert!(hit.is_none());
    }

    #[test]
    fn hit_center_front_face() {
        let sphere = unit_sphere(Point3::ORIGIN);

        let ray = Ray::new(Point3::new(0.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0));

        let hit = sphere
            .check_intersection(&ray, Interval::new(0.001, f64::INFINITY))
            .expect("Ray should hit sphere");

        // Hit point should be at z = -1
        assert!((hit.point - Point3::new(0.0, 0.0, -1.0)).length() < 1e-6);

        // Normal should point straight back toward camera
        assert!((hit.normal.0 - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-6);

        // Normal must be unit length
        assert!((hit.normal.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn hit_offset_point() {
        let sphere = unit_sphere(Point3::ORIGIN);

        let ray = Ray::new(
            Point3::new(0.0, 0.0, -3.0),
            Vec3::new(0.0, 0.2, 1.0).normalized(),
        );

        let hit = sphere
            .check_intersection(&ray, Interval::new(0.001, f64::INFINITY))
            .expect("Ray should hit sphere");

        // Normal must match radial direction
        let expected = (hit.point - Point3::ORIGIN).normalized();

        assert!((hit.normal.0 - expected).length() < 1e-6);
    }

    #[test]
    fn sphere_front_face() {
        let sphere = sphere(Point3::ORIGIN, 10.0);

        let ray_inner = Ray::new(Point3::ORIGIN, Vec3::new(1.0, 0.0, 0.0));

        let hit = sphere
            .check_intersection(&ray_inner, Interval::new(0.001, f64::INFINITY))
            .expect("Ray should hit the sphere");

        // A ray coming from within the sphere should have the correct `front_face` property
        assert!(!hit.front_face);

        let ray_outer = Ray::new(Point3::new(-15.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

        let hit = sphere
            .check_intersection(&ray_outer, Interval::new(0.001, f64::INFINITY))
            .expect("Ray should hit the sphere");

        // A ray comming from outside
        assert!(hit.front_face);
    }
}
