mod color;
mod config;
mod math;
mod progressbar;
mod rendering;

use std::{env, fs, process};

use crate::config::*;
use crate::math::interval::Interval;
use crate::math::*;
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

    // World
    let mut world = World::new();
    world.add(Box::new(geometry::Sphere {
        center: Point3::new(0.0, 0.0, -1.0),
        radius: 0.5,
    }));

    world.add(Box::new(geometry::Sphere {
        center: Point3::new(0.0, -100.5, -1.0),
        radius: 100.0,
    }));

    let mut rng = rand::rng();

    let camera = Camera::new(config.camera.aspect_ratio, config.camera.image_width);

    let mut sampler = RandomSampler::new(&mut rng);

    let renderer = Renderer::new(
        config.renderer.samples_per_pixel,
        config.renderer.max_bounces,
    );

    renderer.render(&world, &camera, &mut sampler);
}
