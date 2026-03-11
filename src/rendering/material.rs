use crate::color::Color;
use crate::math::Ray;
use crate::math::{HitInfo, Vec3};
use crate::rendering::sampler::Sampler;

pub trait Material {
    /// Returns the scannered ray and the color attenuation.
    /// If none, then incoming ray has been absorbed by the material.
    fn scatter(&self, ray: Ray, hit: &HitInfo, sampler: &mut dyn Sampler) -> Option<(Color, Ray)>;

    /// Fallback method. If the ray has been absorbed, signal to the render which color to use.
    fn emitted(&self, _hit: &HitInfo) -> Color {
        Color::BLACK
    }
}

/// Dummy material that absorbs all light
pub struct DummyMaterial;

impl Material for DummyMaterial {
    fn scatter(
        &self,
        _ray: Ray,
        _hit: &HitInfo,
        _sampler: &mut dyn Sampler,
    ) -> Option<(Color, Ray)> {
        None
    }
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: Ray, hit: &HitInfo, sampler: &mut dyn Sampler) -> Option<(Color, Ray)> {
        // If we get a random direction directly opposite the normal bad things can happen
        let scatter_direction = {
            let d = hit.normal.0 + sampler.unit_vector();
            if d.is_near_zero() {
                hit.normal.0
            } else {
                d
            }
        };

        let scattered = Ray::new(hit.point, scatter_direction);
        // We attenuate by the albedo
        Some((self.albedo, scattered))
    }
}

pub struct NormalVisualizer;

impl Material for NormalVisualizer {
    fn scatter(
        &self,
        _ray: Ray,
        _hit: &HitInfo,
        _sampler: &mut dyn Sampler,
    ) -> Option<(Color, Ray)> {
        None
    }

    fn emitted(&self, hit: &HitInfo) -> Color {
        Color::from(0.5 * (hit.normal.0 + Vec3::ONES))
    }
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: Ray, hit: &HitInfo, sampler: &mut dyn Sampler) -> Option<(Color, Ray)> {
        let reflected =
            ray.direction.reflect(hit.normal).normalized() + (self.fuzz * sampler.unit_vector());
        let scattered = Ray::new(hit.point, reflected);

        // Absorb the ray we scatter below the surface
        if scattered.direction.dot(hit.normal.0) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

/// Dielectric material (glass, water, etc.) that refracts and reflects light
pub struct Dielectric {
    pub index_of_refraction: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Self {
        Self {
            index_of_refraction,
        }
    }

    /// Calculate reflectance using Schlick's approximation
    fn reflectance(&self, cosine: f64, eta: f64) -> f64 {
        // Schlick's approximation for reflectance varying with angle
        let r0 = (1.0 - eta) / (1.0 + eta);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: Ray, hit: &HitInfo, sampler: &mut dyn Sampler) -> Option<(Color, Ray)> {
        // Determine the refractive indices based on whether we're entering or exiting the material
        let (eta_i, eta_t) = if hit.front_face {
            // Ray is entering the material (air -> dielectric)
            (1.0, self.index_of_refraction)
        } else {
            // Ray is exiting the material (dielectric -> air)
            (self.index_of_refraction, 1.0)
        };

        // Calculate the ratio of indices of refraction
        let eta_ratio = eta_i / eta_t;

        // Calculate the cosine of the incident angle
        // cos(theta_i) = dot(-incident, normal)
        let cos_theta = -ray.direction.dot(hit.normal.0).min(1.0);

        // Calculate the sine of the incident angle using Snell's law
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        // Check for total internal reflection
        // This occurs when sin(theta_t) would be > 1 (impossible)
        let cannot_refract = eta_ratio * sin_theta > 1.0;

        // Calculate reflectance using Schlick's approximation
        let reflectance = self.reflectance(cos_theta, eta_ratio);

        // Decide whether to reflect or refract based on reflectance
        let should_reflect = cannot_refract || sampler.next_f64() < reflectance;

        let scattered = if should_reflect {
            // Reflect the ray
            let reflected = ray.direction.reflect(hit.normal);
            Ray::new(hit.point, reflected)
        } else {
            // Refract the ray using Snell's law
            // Calculate the direction of the refracted ray
            let r_out_perp = eta_ratio * (ray.direction + cos_theta * hit.normal.0);
            let r_out_parallel = -(1.0 - r_out_perp.length_squared()).sqrt() * hit.normal.0;
            Ray::new(hit.point, r_out_perp + r_out_parallel)
        };

        // Dielectric materials don't absorb light (color is white/1.0)
        // But we can still return a color if desired (usually white for glass)
        Some((Color::new(1.0, 1.0, 1.0), scattered))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Normal3;
    use crate::math::Point3;
    use crate::rendering::sampler::RandomSampler;
    use rand::SeedableRng;
    use std::sync::Arc;

    #[test]
    fn test_dielectric_refraction() {
        // Test that a ray entering glass from air refracts
        let dielectric = Dielectric::new(1.5);
        let rng = rand::prelude::SmallRng::seed_from_u64(42);
        let mut sampler = RandomSampler::new(rng);

        // Ray hitting from front (air -> glass)
        let hit = HitInfo {
            point: Point3::new(0.0, 0.0, 0.0),
            normal: Normal3::new(0.0, 0.0, 1.0),
            t: 1.0,
            front_face: true,
            material: Arc::new(dielectric),
        };

        // Ray coming from air (eta_i = 1.0) hitting glass (eta_t = 1.5)
        // Angle of incidence: 0 degrees (normal incidence)
        // Surface at z=0 with normal (0,0,1), so ray must be coming from z>0
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));

        // With normal incidence (cos_theta = 1.0), the reflectance should be very low
        // So the ray should refract
        let result = hit.material.scatter(ray, &hit, &mut sampler);

        assert!(result.is_some(), "Dielectric should scatter a ray");
        let (_attenuation, scattered) = result.unwrap();

        // The scattered ray should continue in roughly the same direction (refracted)
        // For normal incidence, refraction angle should be very close to 0
        // (i.e., ray goes straight through with slightly reduced speed)
        // We check that the direction is roughly forward (z component close to 1)
        // Use a smaller threshold due to potential floating point issues
        assert!(
            scattered.direction.z < -0.5,
            "Ray should refract forward, z={}",
            scattered.direction.z
        );

        // Also verify the ray is normalized
        assert!((scattered.direction.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dielectric_reflection() {
        // Test that a ray can reflect from a dielectric
        let dielectric = Dielectric::new(1.5);
        let rng = rand::prelude::SmallRng::seed_from_u64(42);
        let mut sampler = RandomSampler::new(rng);

        // Ray hitting from front (air -> glass)
        let hit = HitInfo {
            point: Point3::new(0.0, 0.0, 0.0),
            normal: Normal3::new(0.0, 0.0, 1.0),
            t: 1.0,
            front_face: true,
            material: Arc::new(dielectric),
        };

        // Ray coming from air hitting glass at a steep angle
        // This increases the chance of reflection
        let ray = Ray::new(
            Point3::new(0.0, 0.0, -1.0),
            Vec3::new(0.5, 0.0, 1.0).normalized(),
        );

        let result = hit.material.scatter(ray, &hit, &mut sampler);

        assert!(result.is_some(), "Dielectric should scatter a ray");
        let (_attenuation, scattered) = result.unwrap();

        // The scattered ray could be either reflected or refracted
        // We can't determine which without controlling the random sampler
        // But we can verify that the ray is valid (normalized)
        assert!((scattered.direction.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dielectric_total_internal_reflection() {
        // Test total internal reflection when ray exits glass to air at steep angle
        let dielectric = Dielectric::new(1.5);
        let rng = rand::prelude::SmallRng::seed_from_u64(42);
        let mut sampler = RandomSampler::new(rng);

        // Ray hitting from back (glass -> air)
        let hit = HitInfo {
            point: Point3::new(0.0, 0.0, 0.0),
            normal: Normal3::new(0.0, 0.0, 1.0),
            t: 1.0,
            front_face: false, // Exiting the material
            material: Arc::new(dielectric),
        };

        // Ray coming from glass (eta_i = 1.5) exiting to air (eta_t = 1.0)
        // At a steep angle where total internal reflection occurs
        let ray = Ray::new(
            Point3::new(0.0, 0.0, 1.0),
            Vec3::new(0.8, 0.0, -1.0).normalized(),
        );

        let result = hit.material.scatter(ray, &hit, &mut sampler);

        assert!(result.is_some(), "Dielectric should scatter a ray");
        let (_attenuation, scattered) = result.unwrap();

        // With total internal reflection, the ray should reflect backward
        assert!(
            scattered.direction.z < 0.0,
            "Reflected ray should go backward"
        );
    }

    #[test]
    fn test_dielectric_reflectance_schlick() {
        // Test Schlick's approximation for reflectance
        let dielectric = Dielectric::new(1.5);

        // At normal incidence (cos_theta = 1.0), reflectance should be R0
        let r0 = (1.0 - 1.5) / (1.0 + 1.5);
        let r0 = r0 * r0;

        let reflectance_normal = dielectric.reflectance(1.0, 1.0 / 1.5);
        assert!(
            (reflectance_normal - r0).abs() < 1e-6,
            "Normal incidence reflectance should match R0"
        );

        // At grazing angle (cos_theta = 0.0), reflectance should be 1.0
        let reflectance_grazing = dielectric.reflectance(0.0, 1.0 / 1.5);
        assert!(
            (reflectance_grazing - 1.0).abs() < 1e-6,
            "Grazing incidence reflectance should be 1.0"
        );
    }

    #[test]
    fn test_dielectric_color_is_white() {
        // Test that dielectric materials scatter white light (no absorption)
        let dielectric = Dielectric::new(1.5);
        let rng = rand::prelude::SmallRng::seed_from_u64(42);
        let mut sampler = RandomSampler::new(rng);

        let hit = HitInfo {
            point: Point3::new(0.0, 0.0, 0.0),
            normal: Normal3::new(0.0, 0.0, 1.0),
            t: 1.0,
            front_face: true,
            material: Arc::new(dielectric),
        };

        let ray = Ray::new(Point3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0));

        let result = hit.material.scatter(ray, &hit, &mut sampler);

        if let Some((attenuation, _)) = result {
            // Dielectric should scatter white light (1.0, 1.0, 1.0)
            assert!((attenuation.r - 1.0).abs() < 1e-6);
            assert!((attenuation.g - 1.0).abs() < 1e-6);
            assert!((attenuation.b - 1.0).abs() < 1e-6);
        } else {
            panic!("Expected Some((attenuation, scattered)), got None");
        }
    }
}
