use std::fs;
use std::{fs::File, iter::zip, path::PathBuf};

use yart::{
    Camera, Renderer, World, image::Image, load_scene_from_config, math::Vec3, sampler::Sampler,
};

pub fn should_update_goldens() -> bool {
    std::env::var("UPDATE_GOLDENS").is_ok()
}

fn golden_image_path(name: &str) -> PathBuf {
    PathBuf::from("golden_images")
        .join("images")
        .join(name.to_owned() + ".ppm")
}

fn load_golden_scene(name: &str) -> (Camera, World, Renderer) {
    let config_path = PathBuf::from("golden_images")
        .join("scenes")
        .join(name.to_owned() + ".toml");
    let contents = fs::read_to_string(config_path).expect("could not load golden scene");
    // let config: yart::Config = toml::from_str(&contents).expect("could not parse config file");
    let config: yart::Config = toml::from_str(&contents).unwrap_or_else(|err| {
        eprintln!("Could not read config: {err}");
        panic!();
    });

    load_scene_from_config(&config)
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

pub fn golden_test(name: &str, sampler: &mut dyn Sampler) {
    let (camera, world, renderer) = load_golden_scene(name);

    let image = renderer.render(&world, &camera, sampler, false);

    let path = golden_image_path(name);
    if should_update_goldens() {
        let mut file = File::create(path).expect("Could not create file");
        image.write_ppm(&mut file);
    } else {
        let mut file = File::open(path).expect("could not open file");
        let expected_image = Image::read_from_ppm(&mut file).expect("could not load image {path}");
        assert_images_are_close(&expected_image, &image);
    }
}
