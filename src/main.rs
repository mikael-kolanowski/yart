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
    println!("yart --editor [config.toml]");
}

fn main() {
    colog::init();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--editor" {
        let config_path = args.get(2).map(|s| s.as_str());

        let editor = {
            if let Some(path) = config_path {
                let config = match read_config(path) {
                    Ok(config) => config,
                    Err(err) => {
                        error!("could not read config: {err}");
                        process::exit(1);
                    }
                };
                yart::editor::Editor::with_config(&config)
            } else {
                yart::editor::Editor::new()
            }
        };

        let options = eframe::NativeOptions::default();
        let _ = eframe::run_native("YART Editor", options, Box::new(|_cc| Ok(Box::new(editor))));
    }

    let config_path = args.get(1).unwrap_or_else(|| {
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
