use rand::Rng;

use crate::math::Vec3;

pub struct RandomSampler<R: Rng> {
    pub rng: R,
}

impl<R: Rng> RandomSampler<R> {
    pub fn new(rng: R) -> Self {
        Self { rng }
    }

    /// Returns a random vector where each component is in the range [0, 1)
    pub fn random_vec3(&mut self) -> Vec3 {
        Vec3::new(self.rng.random(), self.rng.random(), self.rng.random())
    }

    /// Returns a random vector with length 1.0
    pub fn random_unit_vector(&mut self) -> Vec3 {
        loop {
            let p = self.random_vec3();
            let length_squared = p.length_squared();
            // Reject vectors in the black hole region, that normalized would yeld
            // [inf, inf, inf]
            if 1e-160 < length_squared && length_squared <= 1.0 {
                return p / length_squared.sqrt();
            }
        }
    }

    pub fn random_unit_vector_on_hemisphere(&mut self, normal: Vec3) -> Vec3 {
        let on_unit_sphere = self.random_unit_vector();
        if on_unit_sphere.dot(normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }

    /// Returns the vector to a random point in the [-0.5, -0.5]-[0.5, 0.5] unit square
    pub fn sample_square(&mut self) -> Vec3 {
        let a: f64 = self.rng.random();
        let b: f64 = self.rng.random();
        Vec3::new(a - 0.5, b - 0.5, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    pub fn random_unit_vector_is_unit() {
        let rng = rand::prelude::SmallRng::seed_from_u64(1337);
        let mut sampler = RandomSampler::new(rng);

        let ten_unit_vectors: Vec<Vec3> = (0..10).map(|_| sampler.random_unit_vector()).collect();

        for unit in &ten_unit_vectors {
            assert!(unit.length() - 1.0 < 1e-6)
        }
    }
}
