mod common;

use std::sync::Arc;

use yart::{
    Camera, World,
    color::Color,
    material::{Lambertian, Metal, NormalVisualizer},
    math::{Sphere, Vec3},
    sampler::RandomSampler,
    sky::LinearGradientSkyBox,
};

use crate::common::golden::golden_test;
use rand::{SeedableRng, rngs::SmallRng};

#[test]
fn red_sphere_head_on() {
    let rng = SmallRng::seed_from_u64(1337);
    let mut sampler = RandomSampler::new(rng);

    let camera = Camera::new(1.0, 32, 90, Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

    let material = Arc::new(Lambertian::new(Color::new(1.0, 0.0, 0.0)));

    let sky = LinearGradientSkyBox {
        from: Color::WHITE,
        to: Color::new(0.5, 0.7, 1.0),
    };

    let sphere = Sphere {
        center: Vec3::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material,
    };

    let world = World {
        objects: vec![Box::new(sphere)],
        skybox: Box::new(sky),
    };

    golden_test("red_sphere", &world, &camera, &mut sampler)
}

#[test]
fn sphere_normals() {
    let rng = SmallRng::seed_from_u64(1337);
    let mut sampler = RandomSampler::new(rng);

    let camera = Camera::new(1.0, 32, 90, Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

    let material = Arc::new(NormalVisualizer);

    let sky = LinearGradientSkyBox {
        from: Color::WHITE,
        to: Color::new(0.5, 0.7, 1.0),
    };

    let sphere = Sphere {
        center: Vec3::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material,
    };

    let world = World {
        objects: vec![Box::new(sphere)],
        skybox: Box::new(sky),
    };

    golden_test("sphere_normals", &world, &camera, &mut sampler)
}

#[test]
fn red_sphere_with_ground_plane() {
    let rng = SmallRng::seed_from_u64(1337);
    let mut sampler = RandomSampler::new(rng);

    let camera = Camera::new(1.0, 32, 90, Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

    let red = Arc::new(Lambertian::new(Color::new(1.0, 0.0, 0.0)));
    let green = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));

    let sky = LinearGradientSkyBox {
        from: Color::WHITE,
        to: Color::new(0.5, 0.7, 1.0),
    };

    let sphere = Sphere {
        center: Vec3::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material: red,
    };

    let ground = Sphere {
        center: Vec3::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: green,
    };

    let world = World {
        objects: vec![Box::new(sphere), Box::new(ground)],
        skybox: Box::new(sky),
    };

    golden_test("sphere_with_ground", &world, &camera, &mut sampler)
}

#[test]
fn matte_and_metal_sphere() {
    let rng = SmallRng::seed_from_u64(1337);
    let mut sampler = RandomSampler::new(rng);

    let camera = Camera::new(1.0, 32, 90, Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

    let red = Arc::new(Lambertian::new(Color::new(1.0, 0.0, 0.0)));
    let metal = Arc::new(Metal::new(Color::WHITE, 0.4));
    let green = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));

    let sky = LinearGradientSkyBox {
        from: Color::WHITE,
        to: Color::new(0.5, 0.7, 1.0),
    };

    let sphere1 = Sphere {
        center: Vec3::new(0.5, 0.0, -1.0),
        radius: 0.5,
        material: red,
    };

    let sphere2 = Sphere {
        center: Vec3::new(-0.5, 0.0, -1.0),
        radius: 0.5,
        material: metal,
    };

    let ground = Sphere {
        center: Vec3::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: green,
    };

    let world = World {
        objects: vec![Box::new(sphere1), Box::new(sphere2), Box::new(ground)],
        skybox: Box::new(sky),
    };

    golden_test("two_spheres", &world, &camera, &mut sampler)
}
