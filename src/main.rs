mod color;
mod config;
mod math;
mod progressbar;
mod rendering;

use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs, process};

use crate::color::Color;
use crate::config::*;
use crate::math::interval::Interval;
use crate::math::*;
use crate::rendering::material::{Lambertian, Metal, NormalVisualizer};
use crate::rendering::sampler::RandomSampler;
use crate::rendering::*;

struct World {
    objects: Vec<Box<dyn Hittable>>,
}

impl World {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    fn from_config(config: &Config) -> Self {
        let mut material_map: HashMap<String, Arc<dyn Material>> = HashMap::new();
        for material_config in &config.materials {
            match material_config {
                MaterialConfig::Lambertian { name, albedo } => {
                    let lamb = Lambertian::new(Color::from(*albedo));
                    material_map.insert(name.clone(), Arc::new(lamb))
                }
                MaterialConfig::Metal { name, albedo, fuzz } => {
                    let metal = Metal::new(Color::from(*albedo), *fuzz);
                    material_map.insert(name.clone(), Arc::new(metal))
                }
                MaterialConfig::NormalVisualization { name } => {
                    let mat = NormalVisualizer;
                    material_map.insert(name.clone(), Arc::new(mat))
                }
            };
        }

        let mut objects: Vec<Box<dyn Hittable>> = Vec::new();
        for object_config in &config.objects {
            match object_config {
                ObjectConfig::Sphere {
                    position,
                    radius,
                    material,
                } => {
                    let material = material_map.get(material).expect("unknown material");
                    objects.push(Box::new(Sphere {
                        center: *position,
                        radius: *radius,
                        material: material.clone(),
                    }));
                }
            }
        }

        World { objects: objects }
    }

    fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for World {
    fn check_intersection(&self, ray: &Ray, ray_t: Interval) -> Option<HitInfo> {
        let mut closest = ray_t.max;
        let mut hit_anything = None;

        for obj in &self.objects {
            if let Some(hit) = obj.check_intersection(ray, Interval::new(ray_t.min, closest)) {
                closest = hit.t;
                hit_anything = Some(hit)
            }
        }
        return hit_anything;
    }
}

fn read_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

fn open_file(path: PathBuf) -> std::io::Result<File> {
    File::create(path)
}

fn print_usage() {
    println!("Usage: ");
    println!("yart <config.toml>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error: no config file supplied");
        print_usage();
        process::exit(1);
    }

    let config_path = &args[1];

    let config = read_config(config_path).unwrap_or_else(|err| {
        eprintln!("Could not read config: {err}");
        process::exit(1);
    });

    let world = World::from_config(&config);

    let mut rng = rand::rng();

    let camera = Camera::new(
        config.camera.aspect_ratio,
        config.camera.image_width,
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
    let mut output_file = open_file(config.image.output).expect("Unable to open output file");
    image.write_ppm(&mut output_file);
}
