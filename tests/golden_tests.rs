mod common;

use yart::sampler::RandomSampler;

use crate::common::golden::golden_test;
use rand::{SeedableRng, rngs::SmallRng};

#[test]
fn red_sphere_head_on() {
    let rng = SmallRng::seed_from_u64(1337);
    let mut sampler = RandomSampler::new(rng);

    golden_test("red_sphere", &mut sampler);
}

#[test]
fn sphere_normals() {
    let rng = SmallRng::seed_from_u64(1337);
    let mut sampler = RandomSampler::new(rng);

    golden_test("sphere_normals", &mut sampler);
}

#[test]
fn red_sphere_with_ground_plane() {
    let rng = SmallRng::seed_from_u64(1337);
    let mut sampler = RandomSampler::new(rng);

    golden_test("sphere_with_ground", &mut sampler)
}

#[test]
fn matte_and_metal_sphere() {
    let rng = SmallRng::seed_from_u64(1337);
    let mut sampler = RandomSampler::new(rng);

    golden_test("two_spheres", &mut sampler)
}
