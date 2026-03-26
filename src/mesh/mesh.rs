use std::io::Read;

use log::{info, warn};

use crate::{
    material::MaterialLibrary,
    math::{Point3, Triangle},
};

pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    /// Parse an OBJ format into a mesh.
    ///
    /// # Arguments
    /// * `reader` OBJ source
    /// * `material_library` the material library to use when resolving materials
    /// * `default_material_id` the ID of the material in `material_library` to use in the absence
    ///     of `usemtl`
    ///
    /// `usemtl` directives are looked up in the provided material library.
    pub fn read_from_obj<R: Read>(
        reader: &mut R,
        material_library: &MaterialLibrary,
        default_material_id: usize,
    ) -> Result<Self, String> {
        let mut contents = String::new();
        reader
            .read_to_string(&mut contents)
            .map_err(|_| "unable to read obj contents")?;

        let mut vertices: Vec<Point3> = Vec::new();
        let mut normal_vertices: Vec<Point3> = Vec::new();
        let mut texture_coordinates: Vec<(f64, f64)> = Vec::new();
        let mut triangles: Vec<Triangle> = Vec::new();

        let mut current_material_id = default_material_id;

        for line in contents.lines() {
            let line = line.trim();
            // Skip comments and empty lines
            if line.starts_with("#") || line.is_empty() {
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

                    let tris = triangulate_fan(&polygon, current_material_id);
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
                    let material_name = args;
                    current_material_id = material_library.lookup_material_id(material_name);
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

fn triangulate_fan(polygon: &[Point3], material_id: usize) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    let n = polygon.len();

    for i in 1..n - 1 {
        let tri = Triangle {
            p1: polygon[0],
            p2: polygon[i],
            p3: polygon[i + 1],
            material_id,
        };
        triangles.push(tri);
    }

    triangles
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{color::Color, material::Lambertian};

    use super::*;

    #[test]
    fn load_obj_single_triangle() {
        // first line of the stanford bunny
        let source = "# Comments should be skipped
                     v -3.4101800e-003 1.3031957e-001 2.1754370e-002
                     v -8.1719160e-002 1.5250145e-001 2.9656090e-002
                     v -3.0543480e-002 1.2477885e-001 1.0983400e-003
                     f 1 2 3";

        let mut cursor = std::io::Cursor::new(source);

        let mesh = Mesh::read_from_obj(&mut cursor, &MaterialLibrary::new(), 0).unwrap();
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

        let mesh = Mesh::read_from_obj(&mut cursor, &MaterialLibrary::new(), 0).unwrap();
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

        let mesh = Mesh::read_from_obj(&mut cursor, &MaterialLibrary::new(), 0).unwrap();
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

        let mesh = Mesh::read_from_obj(&mut cursor, &MaterialLibrary::new(), 0).unwrap();
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

        let mesh = Mesh::read_from_obj(&mut cursor, &MaterialLibrary::new(), 0).unwrap();
        assert_eq!(mesh.triangles.len(), 1);
    }

    #[test]
    fn load_mesh_with_different_materials() {
        let mut material_library = MaterialLibrary::new();
        let red = Arc::new(Lambertian::new(Color::new(1.0, 0.0, 0.0)));
        let blue = Arc::new(Lambertian::new(Color::new(0.0, 0.0, 1.0)));
        material_library.register_material("red", red);
        material_library.register_material("blue", blue);

        let source = "# test_usemtl.obj
                      o TestMesh
                      v 0.0 0.0 0.0
                      v 1.0 0.0 0.0
                      v 0.0 1.0 0.0

                      v 0.0 0.0 1.0
                      v 1.0 0.0 1.0
                      v 0.0 1.0 1.0

                      v 0.0 0.0 2.0
                      v 1.0 0.0 2.0
                      v 0.0 1.0 2.0

                      f 1 2 3

                      usemtl red
                      f 4 5 6

                      usemtl blue
                      f 7 8 9";

        let mut cursor = std::io::Cursor::new(source);

        let mesh = Mesh::read_from_obj(&mut cursor, &material_library, 0).unwrap();
        assert_eq!(mesh.triangles.len(), 3);

        // Default material
        assert_eq!(mesh.triangles[0].material_id, 0);
        assert_eq!(
            mesh.triangles[1].material_id,
            material_library.lookup_material_id("red")
        );
        assert_eq!(
            mesh.triangles[2].material_id,
            material_library.lookup_material_id("blue")
        );
    }
}
