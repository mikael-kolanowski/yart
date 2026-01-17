use rand::Rng;

use crate::math::Vec3;

pub struct Sampler<R: Rng> {
    pub rng: R,
}

impl<R: Rng> Sampler<R> {
    pub fn new(rng: R) -> Self {
        Self { rng }
    }

    /// Returns the vector to a random point in the [-0.5, -0.5]-[0.5, 0.5] unit square
    pub fn sample_square(&mut self) -> Vec3 {
        let a: f64 = self.rng.random();
        let b: f64 = self.rng.random();
        Vec3::new(a - 0.5, b - 0.5, 0.0)
    }
}
