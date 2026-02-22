mod color;
mod config;
mod math;
mod progressbar;
mod rendering;
mod world;

use std::fs::File;
use std::{env, fs, process};

use crate::config::*;
use crate::rendering::sampler::RandomSampler;
use crate::rendering::*;
use crate::world::World;

fn read_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

fn print_usage() {
    println!("Usage: ");
    println!("yart <config.toml>");
}

fn main() {
    let config_path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Error: no config file supplied");
        print_usage();
        process::exit(1);
    });

    let config = read_config(&config_path).unwrap_or_else(|err| {
        eprintln!("Could not read config: {err}");
        process::exit(1);
    });

    let world = World::from_config(&config);

    let mut rng = rand::rng();

    let camera = Camera::new(
        config.camera.aspect_ratio,
        config.image.width,
        config.camera.field_of_view,
        config.camera.position,
        config.camera.look_at,
    );

    let mut sampler = RandomSampler::new(&mut rng);

    let renderer = Renderer::new(
        config.renderer.samples_per_pixel,
        config.renderer.max_bounces,
    );

    let image = renderer.render(&world, &camera, &mut sampler, true);
    let mut output_file = File::create(config.image.output).expect("Unable to open output file");
    image.write_ppm(&mut output_file);
}
