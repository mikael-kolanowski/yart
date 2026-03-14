use std::path::PathBuf;

use crate::{
    config::*,
    math::{Point3, Vec3},
};

fn material_label(mat: &MaterialConfig) -> String {
    match mat {
        MaterialConfig::Lambertian { .. } => "lambertian".into(),
        MaterialConfig::Metal { .. } => "metal".into(),
        MaterialConfig::NormalVisualization { .. } => "normal_vis".into(),
    }
}

pub fn new_material_name(
    selected_material_label: &str,
    existing_materials: &[MaterialConfig],
) -> String {
    let n = existing_materials
        .iter()
        .filter(|mat| material_label(mat).eq(selected_material_label))
        .count();
    format!("{selected_material_label}_{n}")
}

pub fn validate_object(obj: &ObjectConfig, materials: &[MaterialConfig]) -> Result<(), String> {
    let material_name = match obj {
        ObjectConfig::Sphere { material, .. } => material,
        ObjectConfig::Triangle { material, .. } => material,
        ObjectConfig::Mesh { material, .. } => material,
    };

    let material_exists = materials.iter().any(|m| m.name() == *material_name);
    if !material_exists {
        return Err("Material not found".to_string());
    }

    if let ObjectConfig::Mesh { path, .. } = obj {
        if path.as_os_str().is_empty() {
            return Err("Mesh path is required".to_string());
        }
    }

    Ok(())
}

pub fn validate_material(mat: &MaterialConfig, existing: &[MaterialConfig]) -> Result<(), String> {
    let name = mat.name();

    if name.is_empty() {
        return Err("Name is required".to_string());
    }

    let name_unique = !existing.iter().any(|m| m.name() == name);
    if !name_unique {
        return Err("Name must be unique".to_string());
    }

    Ok(())
}

pub fn default_config() -> Config {
    Config {
        camera: crate::config::CameraConfig {
            aspect_ratio: 16.0 / 9.0,
            field_of_view: 90,
            position: Point3::new(-1.0, 1.0, 1.0),
            look_at: Point3::new(0.0, 0.0, -1.0),
        },
        renderer: crate::config::RendererConfig {
            samples_per_pixel: 20,
            max_bounces: 10,
        },
        image: crate::config::ImageConfig {
            width: 400,
            output: PathBuf::from("output.ppm"),
        },
        materials: vec![MaterialConfig::Lambertian {
            name: "matte".to_string(),
            albedo: Vec3::new(0.5, 0.5, 0.5),
        }],
        objects: vec![ObjectConfig::Sphere {
            position: Vec3::new(0.0, 0.0, -1.0),
            radius: 1.0,
            material: "matte".to_string(),
        }],
        sky: SkyConfig::LinearGradient {
            from: Vec3::new(1.0, 1.0, 1.0),
            to: Vec3::new(0.5, 0.7, 1.0),
        },
    }
}
