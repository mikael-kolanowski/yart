use crate::config::*;

pub fn material_label(mat: &MaterialConfig) -> String {
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
