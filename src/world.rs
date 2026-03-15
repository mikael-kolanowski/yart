use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::{collections::HashMap, sync::Arc};

use log::{info, warn};

use crate::color::Color;
use crate::math::{Point3, Sphere, Triangle, geometry::Hittables};
use crate::mesh::Mesh;
use crate::rendering::material::{Dielectric, DummyMaterial, Lambertian, Metal, NormalVisualizer};
use crate::rendering::sky::SkyBox;
use crate::{math::Hittable, rendering::Material};

use crate::config::{Config, MaterialConfig, ObjectConfig, SkyConfig};
use crate::rendering::sky::{LinearGradientSkyBox, SolidColorSkyBox};

pub struct World {
    pub objects: Hittables,
    pub skybox: Box<dyn SkyBox>,
}

impl World {
    pub fn from_config(config: &Config, asset_base_path: &Path) -> Self {
        let mut material_map: HashMap<String, Arc<dyn Material>> = HashMap::new();
        let fallback_material: Arc<dyn Material> = Arc::new(DummyMaterial {});
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
                MaterialConfig::Dielectric { name, ior } => {
                    let dielectric = Dielectric::new(*ior);
                    material_map.insert(name.clone(), Arc::new(dielectric))
                }
            };
        }

        let mut objects = Hittables::new();
        for object_config in &config.objects {
            match object_config {
                ObjectConfig::Sphere {
                    position,
                    radius,
                    material,
                } => {
                    let material = material_map.get(material).unwrap_or_else(|| {
                        warn!("material '{material}' could not be resolved");
                        &fallback_material
                    });
                    objects.add(Box::new(Sphere {
                        center: Point3(*position),
                        radius: *radius,
                        material: material.clone(),
                    }));
                }
                ObjectConfig::Triangle {
                    p1,
                    p2,
                    p3,
                    material,
                } => {
                    let material = material_map.get(material).unwrap_or_else(|| {
                        warn!("material '{material}' could not be resolved");
                        &fallback_material
                    });
                    objects.add(Box::new(Triangle {
                        p1: *p1,
                        p2: *p2,
                        p3: *p3,
                        material: material.clone(),
                    }));
                }
                ObjectConfig::Mesh { path, material } => {
                    let material = material_map.get(material).unwrap_or_else(|| {
                        warn!("material {material} could not be resolved");
                        &fallback_material
                    });
                    let asset_path = resolve_relative_path(asset_base_path, path);
                    match load_mesh_from_path(&asset_path, material.clone()) {
                        Err(message) => {
                            eprintln!("{message}");
                        }
                        Ok(mesh) => {
                            for tri in mesh.triangles {
                                objects.add(Box::new(tri));
                            }
                        }
                    }
                }
            }
        }

        info!("world bounding box: {:?}", objects.bounding_box());
        let skybox = build_skybox(&config.sky);

        World { objects, skybox }
    }
}

fn resolve_relative_path(base: &Path, path: &PathBuf) -> PathBuf {
    if path.is_absolute() {
        path.clone()
    } else {
        base.join(path)
    }
}

fn load_mesh_from_path(path: &PathBuf, material: Arc<dyn Material>) -> Result<Mesh, String> {
    let mut file =
        File::open(&path).map_err(|_| format!("Warning: could not open file {:?}", path))?;
    let mesh = Mesh::read_from_obj(&mut file, material.clone())?;
    Ok(mesh)
}

fn build_skybox(config: &SkyConfig) -> Box<dyn SkyBox> {
    match config {
        SkyConfig::LinearGradient { from, to } => Box::new(LinearGradientSkyBox {
            from: Color::from(*from),
            to: Color::from(*to),
        }),
        SkyConfig::Solid { color } => Box::new(SolidColorSkyBox {
            color: Color::from(*color),
        }),
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            objects: Hittables::new(),
            skybox: Box::new(SolidColorSkyBox {
                color: Color::WHITE,
            }),
        }
    }
}
