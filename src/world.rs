use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::{collections::HashMap, sync::Arc};

use log::{info, warn};

use crate::color::Color;
use crate::math::BVH;
use crate::math::Intersect;
use crate::math::Primitive;
use crate::math::Ray;
use crate::math::{Point3, Sphere, Triangle, interval::Interval};
use crate::mesh::Mesh;
use crate::rendering::Material;
use crate::rendering::material::{Dielectric, DummyMaterial, Lambertian, Metal, NormalVisualizer};
use crate::rendering::sky::SkyBox;

use crate::config::{Config, MaterialConfig, ObjectConfig, SkyConfig};
use crate::rendering::sky::{LinearGradientSkyBox, SolidColorSkyBox};

pub struct SceneObject {
    pub id: usize,
    pub primitive: Primitive,
    pub material_id: usize,
}

pub struct World {
    bvh: BVH,
    skybox: Box<dyn SkyBox>,
    materials: Vec<Arc<dyn Material>>,
}

impl World {
    pub fn from_config(config: &Config, asset_base_path: &Path) -> Self {
        let mut materials: Vec<Arc<dyn Material>> = Vec::new();
        let mut material_name_to_id: HashMap<String, usize> = HashMap::new();

        let fallback_material: Arc<dyn Material> = Arc::new(DummyMaterial {});
        materials.push(fallback_material.clone());
        const FALLBACK_MATERIAL_ID: usize = 0;

        let mut register_material = |name: &String, mat: Arc<dyn Material>| {
            material_name_to_id.insert(name.clone(), materials.len());
            materials.push(mat);
        };

        for material_config in &config.materials {
            match material_config {
                MaterialConfig::Lambertian { name, albedo } => {
                    let lamb = Lambertian::new(Color::from(*albedo));
                    register_material(name, Arc::new(lamb));
                }
                MaterialConfig::Metal { name, albedo, fuzz } => {
                    let metal = Metal::new(Color::from(*albedo), *fuzz);
                    register_material(name, Arc::new(metal));
                }
                MaterialConfig::NormalVisualization { name } => {
                    let mat = NormalVisualizer;
                    register_material(name, Arc::new(mat));
                }
                MaterialConfig::Dielectric { name, ior } => {
                    let dielectric = Dielectric::new(*ior);
                    register_material(name, Arc::new(dielectric));
                }
            };
        }

        let material_name_to_id = material_name_to_id;
        let lookup_material_id = |name: &String| {
            material_name_to_id.get(name).unwrap_or_else(|| {
                warn!("material '{name}' could not be resolved'");
                &FALLBACK_MATERIAL_ID
            })
        };

        let mut primitives: Vec<Primitive> = Vec::new();
        for object_config in &config.objects {
            match object_config {
                ObjectConfig::Sphere {
                    position,
                    radius,
                    material,
                } => {
                    let material_id = lookup_material_id(material);
                    let primitive = Primitive::Sphere(Sphere {
                        center: Point3(*position),
                        radius: *radius,
                        material_id: *material_id,
                    });
                    primitives.push(primitive);
                }
                ObjectConfig::Triangle {
                    p1,
                    p2,
                    p3,
                    material,
                } => {
                    let material_id = lookup_material_id(material);
                    let primitive = Primitive::Triangle(Triangle {
                        p1: *p1,
                        p2: *p2,
                        p3: *p3,
                        material_id: *material_id,
                    });
                    primitives.push(primitive);
                }
                ObjectConfig::Mesh { path, material } => {
                    let material_id = lookup_material_id(material);
                    let asset_path = resolve_relative_path(asset_base_path, path);
                    match load_mesh_from_path(&asset_path, *material_id) {
                        Err(message) => {
                            eprintln!("{message}");
                        }
                        Ok(mesh) => {
                            for tri in mesh.triangles {
                                let primitive = Primitive::Triangle(tri);
                                primitives.push(primitive);
                            }
                        }
                    }
                }
            }
        }

        let skybox = build_skybox(&config.sky);

        let n_objects = primitives.len();
        let n_materials = materials.len();
        let bvh = BVH::build(primitives);

        // Don't include the fallback meterial in the count
        info!(
            "constructed scene: {n_objects} objects, {} materials",
            n_materials - 1
        );

        World {
            bvh,
            skybox,
            materials,
        }
    }

    pub fn lookup_material(&self, id: usize) -> Arc<dyn Material> {
        self.materials.get(id).unwrap_or(&self.materials[0]).clone()
    }

    /// The color to return when the ray does not hit an object in the scene
    pub fn sky(&self, ray: Ray) -> Color {
        self.skybox.color(ray)
    }
}

fn resolve_relative_path(base: &Path, path: &PathBuf) -> PathBuf {
    if path.is_absolute() {
        path.clone()
    } else {
        base.join(path)
    }
}

fn load_mesh_from_path(path: &PathBuf, material_id: usize) -> Result<Mesh, String> {
    let mut file = File::open(&path).map_err(|_| format!("could not open file {:?}", path))?;
    let mesh = Mesh::read_from_obj(&mut file, material_id)?;
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

impl Intersect for World {
    fn intersect(&self, ray: &Ray, interval: Interval) -> Option<crate::math::Hit> {
        self.bvh.intersect(ray, interval)
    }

    fn bounding_box(&self) -> crate::math::AABB {
        self.bvh.bounding_box()
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            bvh: BVH::build(Vec::new()),
            skybox: Box::new(SolidColorSkyBox {
                color: Color::WHITE,
            }),
            materials: Vec::new(),
        }
    }
}
