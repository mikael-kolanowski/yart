use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use log::info;

use crate::color::Color;
use crate::material::MaterialLibrary;
use crate::math::BVH;
use crate::math::Intersect;
use crate::math::Primitive;
use crate::math::Ray;
use crate::math::{Point3, Sphere, Triangle, interval::Interval};
use crate::mesh::Mesh;
use crate::rendering::Material;
use crate::rendering::material::{Dielectric, Lambertian, Metal, NormalVisualizer};
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
    material_library: MaterialLibrary,
}

impl World {
    pub fn from_config(config: &Config, asset_base_path: &Path) -> Self {
        let material_library = Self::build_material_library(&config);

        let mut primitives: Vec<Primitive> = Vec::new();
        for object_config in &config.objects {
            match object_config {
                ObjectConfig::Sphere {
                    position,
                    radius,
                    material,
                } => {
                    let material_id = material_library.lookup_material_id(material);
                    let primitive = Primitive::Sphere(Sphere {
                        center: Point3(*position),
                        radius: *radius,
                        material_id: material_id,
                    });
                    primitives.push(primitive);
                }
                ObjectConfig::Triangle {
                    p1,
                    p2,
                    p3,
                    material,
                } => {
                    let material_id = material_library.lookup_material_id(material);
                    let primitive = Primitive::Triangle(Triangle {
                        p1: *p1,
                        p2: *p2,
                        p3: *p3,
                        material_id: material_id,
                    });
                    primitives.push(primitive);
                }
                ObjectConfig::Mesh { path, material } => {
                    let material_id = material_library.lookup_material_id(material);
                    let asset_path = resolve_relative_path(asset_base_path, path);
                    match load_mesh_from_path(&asset_path, &material_library, material_id) {
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
        let n_materials = material_library.size();
        let bvh = BVH::build(primitives);

        info!(
            "constructed scene: {n_objects} objects, {} materials",
            n_materials
        );

        World {
            bvh,
            skybox,
            material_library,
        }
    }

    fn build_material_library(config: &Config) -> MaterialLibrary {
        let mut material_library = MaterialLibrary::new();

        for material_config in &config.materials {
            match material_config {
                MaterialConfig::Lambertian { name, albedo } => {
                    let lamb = Lambertian::new(Color::from(*albedo));
                    material_library.register_material(name, Arc::new(lamb));
                }
                MaterialConfig::Metal { name, albedo, fuzz } => {
                    let metal = Metal::new(Color::from(*albedo), *fuzz);
                    material_library.register_material(name, Arc::new(metal));
                }
                MaterialConfig::NormalVisualization { name } => {
                    let mat = NormalVisualizer;
                    material_library.register_material(name, Arc::new(mat));
                }
                MaterialConfig::Dielectric { name, ior } => {
                    let dielectric = Dielectric::new(*ior);
                    material_library.register_material(name, Arc::new(dielectric));
                }
            };
        }

        material_library
    }

    pub fn lookup_material(&self, id: usize) -> Arc<dyn Material> {
        self.material_library.lookup_material(id)
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

fn load_mesh_from_path(
    path: &PathBuf,
    material_library: &MaterialLibrary,
    material_id: usize,
) -> Result<Mesh, String> {
    let mut file = File::open(&path).map_err(|_| format!("could not open file {:?}", path))?;
    let mesh = Mesh::read_from_obj(&mut file, material_library, material_id)?;
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
            material_library: MaterialLibrary::new(),
        }
    }
}
