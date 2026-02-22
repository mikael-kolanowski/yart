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
        return None;
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
            let d = hit.normal + sampler.unit_vector();
            if d.is_near_zero() { hit.normal } else { d }
        };

        let scattered = Ray::new(hit.location, scatter_direction);
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
        Color::from(0.5 * (hit.normal + Vec3::ONES))
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
        let scattered = Ray::new(hit.location, reflected);

        // Absorb the ray we scatter below the surface
        if scattered.direction.dot(hit.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}
