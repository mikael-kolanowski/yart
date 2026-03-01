use std::{io::Read, sync::Arc};

use crate::{
    Material,
    math::{Hittable, Point3, Triangle, interval::Interval},
};

pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn read_from_obj<R: Read>(
        reader: &mut R,
        material: Arc<dyn Material>,
    ) -> Result<Self, &'static str> {
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .map_err(|_| "unable to read obj contents")?;

        let mut vertices: Vec<Point3> = Vec::new();
        let mut triangles: Vec<Triangle> = Vec::new();

        for line in contents.lines() {
            let line = line.trim();
            // Skip comments
            if line.starts_with("#") {
                continue;
            }
            let (directive, args) = line.split_once(" ").ok_or("badly formatted line")?;
            match directive {
                "v" => {
                    let parts: Vec<&str> = args.split(" ").collect();
                    let point = {
                        let x = try_parse_f64(parts[0])?;
                        let y = try_parse_f64(parts[1])?;
                        let z = try_parse_f64(parts[2])?;
                        Point3::new(x, y, z)
                    };
                    vertices.push(point);
                }
                "f" => {
                    let parts: Vec<&str> = args.split(" ").collect();
                    let p1 = try_parse_usize(parts[0])?;
                    let p2 = try_parse_usize(parts[1])?;
                    let p3 = try_parse_usize(parts[2])?;

                    // OBJ starts indexing at 1
                    let p1 = vertices
                        .get(p1 - 1)
                        .ok_or("unable to get vertex at index")?;
                    let p2 = vertices
                        .get(p2 - 1)
                        .ok_or("unable to get vertex at index")?;
                    let p3 = vertices
                        .get(p3 - 1)
                        .ok_or("unable to get vertex at index")?;

                    let tri = Triangle {
                        p1: *p1,
                        p2: *p2,
                        p3: *p3,
                        material: material.clone(),
                    };
                    triangles.push(tri);
                }
                _ => {
                    eprintln!("unknown directive");
                }
            }
        }

        eprintln!("Loaded mesh: {} triangles", triangles.len());
        Ok(Self { triangles })
    }
}

impl Hittable for Mesh {
    fn check_intersection(
        &self,
        ray: &crate::math::Ray,
        ray_t: crate::math::interval::Interval,
    ) -> Option<crate::math::HitInfo> {
        let mut closest = ray_t.max;
        let mut hit_anything = None;
        for triangle in &self.triangles {
            if let Some(hit) = triangle.check_intersection(ray, Interval::new(ray_t.min, closest)) {
                closest = hit.t;
                hit_anything = Some(hit);
            }
        }
        hit_anything
    }
}

fn try_parse_f64(s: &str) -> Result<f64, &'static str> {
    s.parse()
        .map_err(|_| "unable to parse float in vertex definition")
}

fn try_parse_usize(s: &str) -> Result<usize, &'static str> {
    s.parse()
        .map_err(|_| "unable to parse index in face definition")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{Material, material::DummyMaterial};

    fn material() -> Arc<dyn Material> {
        Arc::new(DummyMaterial)
    }

    #[test]
    fn load_obj_single_triangle() {
        // first line of the stanford bunny
        let source = "# Comments should be skipped
                     v -3.4101800e-003 1.3031957e-001 2.1754370e-002
                     v -8.1719160e-002 1.5250145e-001 2.9656090e-002
                     v -3.0543480e-002 1.2477885e-001 1.0983400e-003
                     f 1 2 3";

        let mut cursor = std::io::Cursor::new(source);

        let mesh = Mesh::read_from_obj(&mut cursor, material()).unwrap();
        assert_eq!(mesh.triangles.len(), 1);
    }
}
