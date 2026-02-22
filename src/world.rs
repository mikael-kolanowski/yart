use std::{collections::HashMap, sync::Arc};

use crate::color::Color;
use crate::math::interval::Interval;
use crate::math::{HitInfo, Ray, Sphere};
use crate::rendering::material::{Lambertian, Metal, NormalVisualizer};
use crate::{math::Hittable, rendering::Material};

use crate::config::{Config, MaterialConfig, ObjectConfig};

pub struct World {
    objects: Vec<Box<dyn Hittable>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn from_config(config: &Config) -> Self {
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
