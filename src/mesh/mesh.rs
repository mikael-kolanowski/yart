use std::{io::Read, sync::Arc};

use log::{info, warn};

use crate::{
    Material,
    math::{Point3, Triangle},
};

pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn read_from_obj<R: Read>(
        reader: &mut R,
        material: Arc<dyn Material>,
    ) -> Result<Self, String> {
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .map_err(|_| "unable to read obj contents")?;

        let mut vertices: Vec<Point3> = Vec::new();
        let mut normal_vertices: Vec<Point3> = Vec::new();
        let mut texture_coordinates: Vec<(f64, f64)> = Vec::new();
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
                    let point = try_parse_point(parts)?;
                    vertices.push(point);
                }
                "vn" => {
                    let parts: Vec<&str> = args.split(" ").collect();
                    let point = try_parse_point(parts)?;
                    normal_vertices.push(point);
                }
                "vt" => {
                    let parts: Vec<&str> = args.split(" ").collect();
                    let u = try_parse_f64(parts[0])?;
                    let v = try_parse_f64(parts.get(1).unwrap_or(&"0"))?;
                    texture_coordinates.push((u, v));
                }
                "f" => {
                    let parts: Vec<&str> = args.split(" ").collect();
                    let indices: Result<Vec<ObjFaceVertex>, _> =
                        parts.iter().map(|p| try_parse_face_vertex(p)).collect();
                    let indices = indices?;

                    if indices.len() < 3 {
                        warn!("face with fewer than 3 vertices, skipping");
                        continue;
                    }

                    let mut polygon: Vec<Point3> = Vec::with_capacity(indices.len());
                    for face_vertex in &indices {
                        let v = *vertices
                            .get(face_vertex.v - 1)
                            .ok_or("unable to get vertex at index")?;
                        polygon.push(v);
                    }

                    let tris = triangulate_fan(&polygon, material.clone());
                    triangles.extend(tris);
                }
                "o" => {
                    continue;
                }
                "s" => {
                    continue;
                }
                "mtllib" => {
                    // We don't support materials defined in the mtl format.
                    continue;
                }
                "usemtl" => {
                    // TODO: link with materials defined in the scene instead of assigning one
                    // material to the whole mesh.
                    let material = args;
                    info!("skipping looking up material: {material}");
                    continue;
                }
                _ => {
                    warn!("unknown directive: {}", directive);
                }
            }
        }

        info!("loaded mesh: {} triangles", triangles.len());
        Ok(Self { triangles })
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

#[derive(Default)]
struct ObjFaceVertex {
    v: usize,
    vt: Option<usize>,
    vn: Option<usize>,
}

fn try_parse_face_vertex(s: &str) -> Result<ObjFaceVertex, &'static str> {
    let parts: Vec<&str> = s.split('/').collect();
    let v = try_parse_usize(parts[0])?;

    let mut face_vertex = ObjFaceVertex {
        v,
        ..Default::default()
    };

    if parts.len() > 1 && !parts[1].is_empty() {
        face_vertex.vt = Some(try_parse_usize(parts[1])?);
    }

    if parts.len() > 2 && !parts[2].is_empty() {
        face_vertex.vn = Some(try_parse_usize(parts[2])?);
    }

    Ok(face_vertex)
}

fn try_parse_point(parts: Vec<&str>) -> Result<Point3, String> {
    let x = try_parse_f64(parts[0])?;
    let y = try_parse_f64(parts[1])?;
    let z = try_parse_f64(parts[2])?;
    Ok(Point3::new(x, y, z))
}

fn triangulate_fan(polygon: &[Point3], material: Arc<dyn Material>) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    let n = polygon.len();

    for i in 1..n - 1 {
        let tri = Triangle {
            p1: polygon[0],
            p2: polygon[i],
            p3: polygon[i + 1],
            material: material.clone(),
        };
        triangles.push(tri);
    }

    triangles
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

    #[test]
    fn load_obj_quad_triangulates() {
        let source = "v 0.0 0.0 0.0
                     v 1.0 0.0 0.0
                     v 1.0 1.0 0.0
                     v 0.0 1.0 0.0
                     f 1 2 3 4";

        let mut cursor = std::io::Cursor::new(source);

        let mesh = Mesh::read_from_obj(&mut cursor, material()).unwrap();
        assert_eq!(mesh.triangles.len(), 2);
    }

    #[test]
    fn load_obj_quads_and_triangles() {
        let source = "v 0.0 0.0 0.0
                     v 1.0 0.0 0.0
                     v 1.0 1.0 0.0
                     v 0.0 1.0 0.0
                     v 2.0 0.5 0.0
                     f 1 2 3 4
                     f 2 5 3";

        let mut cursor = std::io::Cursor::new(source);

        let mesh = Mesh::read_from_obj(&mut cursor, material()).unwrap();
        assert_eq!(mesh.triangles.len(), 3);
    }

    #[test]
    fn load_obj_face_with_slash_notation() {
        let source = "v 0.0 0.0 0.0
                     v 1.0 0.0 0.0
                     v 1.0 1.0 0.0
                     v 0.0 1.0 0.0
                     vt 0.0 0.0
                     vt 1.0 0.0
                     vt 1.0 1.0
                     vt 0.0 1.0
                     vn 0.0 0.0 1.0
                     vn 0.0 0.0 1.0
                     vn 0.0 0.0 1.0
                     vn 0.0 0.0 1.0
                     f 1/1/1 2/2/2 3/3/3
                     f 1/1/1 3/3/3 4/4/4";

        let mut cursor = std::io::Cursor::new(source);

        let mesh = Mesh::read_from_obj(&mut cursor, material()).unwrap();
        assert_eq!(mesh.triangles.len(), 2);
    }

    #[test]
    fn load_obj_face_vertex_normal_only() {
        let source = "v 0.0 0.0 0.0
                     v 1.0 0.0 0.0
                     v 1.0 1.0 0.0
                     vn 0.0 0.0 1.0
                     vn 0.0 0.0 1.0
                     vn 0.0 0.0 1.0
                     f 1//1 2//2 3//3";

        let mut cursor = std::io::Cursor::new(source);

        let mesh = Mesh::read_from_obj(&mut cursor, material()).unwrap();
        assert_eq!(mesh.triangles.len(), 1);
    }
}
