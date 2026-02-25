use std::{fs::File, iter::zip, path::PathBuf};

use yart::{Renderer, image::Image, math::Vec3, sampler::Sampler};

pub fn should_update_goldens() -> bool {
    std::env::var("UPDATE_GOLDENS").is_ok()
}

pub fn golden_path(name: &str) -> PathBuf {
    PathBuf::from("golden_images").join(name.to_owned() + ".ppm")
}

fn assert_images_are_close(expected: &Image, actual: &Image) {
    assert_eq!(actual.width, expected.width);
    assert_eq!(actual.height, expected.height);

    let threshold = 1e-4;
    let total_squared_error: f64 = zip(actual.pixels.iter(), expected.pixels.iter())
        .map(|(&actual, &expected)| actual - expected)
        .map(|delta| Vec3::new(delta.r, delta.g, delta.b).length_squared())
        .sum();

    let mse = total_squared_error / (actual.width as f64 * actual.height as f64);
    assert!(
        mse < threshold,
        "expected MSE {} to be less than threshold {}",
        mse,
        threshold
    );
}

pub fn golden_test(
    name: &str,
    world: &yart::World,
    camera: &yart::Camera,
    sampler: &mut dyn Sampler,
) {
    let renderer = Renderer::new(32, 32);

    let image = renderer.render(&world, &camera, sampler, false);

    let path = golden_path(name);
    if should_update_goldens() {
        let mut file = File::create(path).expect("Could not create file");
        image.write_ppm(&mut file);
    } else {
        let mut file = File::open(path).expect("could not open file");
        let expected_image = Image::read_from_ppm(&mut file).expect("could not load image {path}");
        assert_images_are_close(&expected_image, &image);
    }
}
