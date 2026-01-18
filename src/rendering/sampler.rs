use crate::math::Vec3;
use rand::Rng;

/// Trait for sampling random values and vectors.
pub trait Sampler {
    /// Returns a random f64 in the range [0, 1)
    fn next_f64(&mut self) -> f64;

    /// Returns a random vector where each component is in the range [0, 1)
    fn vec3(&mut self) -> Vec3 {
        Vec3::new(self.next_f64(), self.next_f64(), self.next_f64())
    }

    /// Returns a random vector with length 1.0
    fn unit_vector(&mut self) -> Vec3 {
        loop {
            let p = self.vec3();
            let length_squared = p.length_squared();
            // Reject vectors in the black hole region, that normalized would yield
            // [inf, inf, inf]
            if 1e-160 < length_squared && length_squared <= 1.0 {
                return p / length_squared.sqrt();
            }
        }
    }

    /// Returns a random unit vector on the hemisphere defined by the normal
    fn unit_vector_on_hemisphere(&mut self, normal: Vec3) -> Vec3 {
        let on_unit_sphere = self.unit_vector();
        if on_unit_sphere.dot(normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }

    /// Returns the vector to a random point in the [-0.5, -0.5]-[0.5, 0.5] unit square
    fn in_square(&mut self) -> Vec3 {
        let a = self.next_f64();
        let b = self.next_f64();
        Vec3::new(a - 0.5, b - 0.5, 0.0)
    }
}

pub struct RandomSampler<R: Rng> {
    pub rng: R,
}

impl<R: Rng> RandomSampler<R> {
    pub fn new(rng: R) -> Self {
        Self { rng }
    }
}

impl<R: Rng> Sampler for RandomSampler<R> {
    fn next_f64(&mut self) -> f64 {
        self.rng.random()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    pub fn random_unit_vector_is_unit() {
        let rng = rand::prelude::SmallRng::seed_from_u64(1337);
        let mut sampler = RandomSampler::new(rng);
        let ten_unit_vectors: Vec<Vec3> = (0..10).map(|_| sampler.unit_vector()).collect();
        for unit in &ten_unit_vectors {
            assert!((unit.length() - 1.0).abs() < 1e-6)
        }
    }

    #[test]
    pub fn trait_object_works() {
        let rng = rand::prelude::SmallRng::seed_from_u64(42);
        let mut sampler: Box<dyn Sampler> = Box::new(RandomSampler::new(rng));

        let vec = sampler.vec3();
        assert!(vec.x >= 0.0 && vec.x < 1.0);
        assert!(vec.y >= 0.0 && vec.y < 1.0);
        assert!(vec.z >= 0.0 && vec.z < 1.0);
    }
}
