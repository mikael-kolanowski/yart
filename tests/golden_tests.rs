mod common;

use yart::{Camera, World, math::Vec3, sampler::RandomSampler};

use crate::common::golden::golden_test;
use rand::{SeedableRng, rngs::SmallRng};

    #[test]
    fn empty_scene() {
        let rng = SmallRng::seed_from_u64(1337);
        let mut sampler = RandomSampler::new(rng);

        let camera = Camera::new(1.0, 32, 90, Vec3::ZERO, Vec3::new(0.0, -1.0, 0.0));

        let world = World::new();

        golden_test("testing", &world, &camera, &mut sampler)
    }