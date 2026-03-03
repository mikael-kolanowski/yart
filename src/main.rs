use std::fs::File;
use std::path::Path;
use std::{env, fs, process};

use log::error;
use log::info;
use yart::load_scene_from_config;

fn read_config(path: &str) -> Result<yart::Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let config: yart::Config = toml::from_str(&contents)?;
    Ok(config)
}

fn print_usage() {
    println!("Usage: ");
    println!("yart <config.toml>");
}

fn main() {
    colog::init();
    let config_path = env::args().nth(1).unwrap_or_else(|| {
        error!("no config file supplied");
        print_usage();
        process::exit(1);
    });

    let config = read_config(&config_path).unwrap_or_else(|err| {
        error!("could not read config: {err}");
        process::exit(1);
    });

    let mut rng = rand::rng();
    let mut sampler = yart::rendering::sampler::RandomSampler::new(&mut rng);

    let asset_base_path = Path::new(&config_path).parent().unwrap();
    let (camera, world, renderer) = load_scene_from_config(&config, &asset_base_path);

    let image = renderer.render(&world, &camera, &mut sampler, true);
    let mut output_file = File::create(&config.image.output).expect("Unable to open output file");
    image.write_ppm(&mut output_file);
    info!("image written to {:?}", config.image.output);
}
