use std::ops::Range;

use super::interval::Interval;
use super::ray::Ray;
use super::vector::{Normal3, Point3, Vec3};

pub struct Hit {
    pub point: Point3,
    pub normal: Normal3,
    pub t: f64,
    pub front_face: bool,
    pub material_id: usize,
}

pub trait Intersect {
    fn intersect(&self, ray: &Ray, interval: Interval) -> Option<Hit>;
    fn bounding_box(&self) -> AABB;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub material_id: usize,
}

impl Intersect for Sphere {
    fn intersect(&self, ray: &Ray, interval: Interval) -> Option<Hit> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let mut root = (h - sqrt_d) / a;
        if !interval.surrounds(root) {
            root = (h + sqrt_d) / a;
            if !interval.surrounds(root) {
                return None;
            }
        }

        let point = ray.at(root);
        let outward_normal = Normal3((point - self.center) / self.radius);
        let mut normal = outward_normal;

        let front_face = {
            if ray.direction.dot(outward_normal.0) > 0.0 {
                normal = -outward_normal;
                false
            } else {
                true
            }
        };

        Some(Hit {
            point,
            normal,
            t: root,
            front_face,
            material_id: self.material_id,
        })
    }

    fn bounding_box(&self) -> AABB {
        let r = Vec3::new(self.radius, self.radius, self.radius);
        AABB::from_extrema(self.center + r, self.center - r)
    }
}

//#[derive(Clone)]
pub struct Triangle {
    pub p1: Point3,
    pub p2: Point3,
    pub p3: Point3,
    pub material_id: usize,
}

impl Intersect for Triangle {
    fn intersect(&self, ray: &Ray, interval: Interval) -> Option<Hit> {
        let edge1 = self.p2 - self.p1;
        let edge2 = self.p3 - self.p1;

        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);

        let eps = 1e-8;
        // Ray parallel to the triangle
        if a.abs() < eps {
            return None;
        }

        let f = 1.0 / a;
        let s = ray.origin - self.p1;

        let u = f * s.dot(h);
        if u < -eps || u > 1.0 + eps {
            return None;
        }

        let q = s.cross(edge1);

        let v = f * ray.direction.dot(q);
        if v < -eps || u + v > 1.0 + eps {
            return None;
        }

        let t = f * edge2.dot(q);

        if !interval.surrounds(t) {
            return None;
        }

        let point = ray.at(t);
        let outward_normal = Normal3(edge1.cross(edge2).normalized());

        let mut normal = outward_normal;
        let front_face = {
            if outward_normal.dot(ray.direction) < 0.0 {
                true
            } else {
                normal = -outward_normal;
                false
            }
        };
        Some(Hit {
            point,
            normal,
            t,
            material_id: self.material_id,
            front_face,
        })
    }

    fn bounding_box(&self) -> AABB {
        let padding = 1e-4;
        let x = Interval::new(
            self.p1.0.x.min(self.p2.0.x).min(self.p3.0.x),
            self.p1.0.x.max(self.p2.0.x).max(self.p3.0.x),
        )
        .expand(padding);

        let y = Interval::new(
            self.p1.0.y.min(self.p2.0.y).min(self.p3.0.y),
            self.p1.0.y.max(self.p2.0.y).max(self.p3.0.y),
        )
        .expand(padding);

        let z = Interval::new(
            self.p1.0.z.min(self.p2.0.z).min(self.p3.0.z),
            self.p1.0.z.max(self.p2.0.z).max(self.p3.0.z),
        )
        .expand(padding);

        AABB { x, y, z }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AABB {
    x: Interval,
    y: Interval,
    z: Interval,
}

impl AABB {
    /// Creates an empty AABB representing no volume.
    pub fn new() -> Self {
        Self {
            x: Interval::EMPTY,
            y: Interval::EMPTY,
            z: Interval::EMPTY,
        }
    }

    /// Creates an AABB from two opposite corner points.
    /// The points need not be in any particular order (min/max are computed automatically).
    fn from_extrema(a: Point3, b: Point3) -> Self {
        let x = {
            if a.0.x <= b.0.x {
                Interval::new(a.0.x, b.0.x)
            } else {
                Interval::new(b.0.x, a.0.x)
            }
        };

        let y = {
            if a.0.y <= b.0.y {
                Interval::new(a.0.y, b.0.y)
            } else {
                Interval::new(b.0.y, a.0.y)
            }
        };

        let z = {
            if a.0.z <= b.0.z {
                Interval::new(a.0.z, b.0.z)
            } else {
                Interval::new(b.0.z, a.0.z)
            }
        };

        Self { x, y, z }
    }

    pub fn longest_axis(&self) -> u32 {
        if self.x.size() > self.y.size() {
            if self.x.size() > self.z.size() { 0 } else { 2 }
        } else {
            if self.y.size() > self.z.size() { 1 } else { 2 }
        }
    }

    /// Combines two AABBs into a single AABB that encompasses both.
    /// The resulting box has intervals spanning from the min of each to the max of each.
    pub fn from_boxes(a: Self, b: Self) -> Self {
        let x = Interval::from(a.x, b.x);
        let y = Interval::from(a.y, b.y);
        let z = Interval::from(a.z, b.z);

        Self { x, y, z }
    }

    fn axis_interval(&self, n: u32) -> Interval {
        match n {
            1 => self.y,
            2 => self.z,
            _ => self.x,
        }
    }

    pub fn hit(&self, ray: &Ray, interval: Interval) -> bool {
        let mut min = interval.min;
        let mut max = interval.max;

        for axis in 0..=2 {
            let interval = self.axis_interval(axis);
            let origin = ray.origin.0.axis(axis);
            let direction = ray.direction.axis(axis);

            // Handle the case where the ray is parallel to the axis
            if direction == 0.0 {
                // If the origin is not between the slab boundaries, return false
                if origin < interval.min || origin > interval.max {
                    return false;
                }
                // Otherwise, the ray is inside the slab, so we don't update t
                continue;
            }

            let inv_d = 1.0 / direction;
            let mut t0 = (interval.min - origin) * inv_d;
            let mut t1 = (interval.max - origin) * inv_d;

            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            if t0 > min {
                min = t0
            }
            if t1 < max {
                max = t1
            }

            if max <= min {
                return false;
            }
        }
        true
    }
}

pub enum Primitive {
    Sphere(Sphere),
    Triangle(Triangle),
}

impl Intersect for Primitive {
    fn intersect(&self, ray: &Ray, interval: Interval) -> Option<Hit> {
        match &self {
            Primitive::Sphere(sphere) => sphere.intersect(ray, interval),
            Primitive::Triangle(triangle) => triangle.intersect(ray, interval),
        }
    }

    fn bounding_box(&self) -> AABB {
        match &self {
            Primitive::Sphere(sphere) => sphere.bounding_box(),
            Primitive::Triangle(triangle) => triangle.bounding_box(),
        }
    }
}

#[derive(Clone, Copy)]
enum BVHNodeKind {
    Inner { left: u32, right: u32 },
    Leaf { start: u32, count: u32 },
}

#[derive(Clone, Copy)]
pub struct BVHNode {
    pub bounding_box: AABB,
    kind: BVHNodeKind,
}

pub struct BVH {
    pub nodes: Vec<BVHNode>,
    pub primitives: Vec<Primitive>,
    root: usize,
}

impl BVH {
    fn build_helper(
        primitives: &mut Vec<Primitive>,
        nodes: &mut Vec<BVHNode>,
        range: Range<usize>,
        depth: u32,
    ) -> usize {
        let count = (range.end - range.start) as u32;
        let mut bounding_box = AABB::new();
        for primitive in &primitives[range.clone()] {
            bounding_box = AABB::from_boxes(bounding_box, primitive.bounding_box());
        }
        // Base case
        if count <= 64 || depth > 36 {
            let node_index = nodes.len();

            let node = BVHNode {
                bounding_box,
                kind: BVHNodeKind::Leaf {
                    start: range.start as u32,
                    count,
                },
            };
            nodes.push(node);
            return node_index;
        }

        let axis = bounding_box.longest_axis();
        let slice = &mut primitives[range.clone()];
        slice.sort_by(|a, b| {
            let a_axis_interval = a.bounding_box().axis_interval(axis);
            let b_axis_interval = b.bounding_box().axis_interval(axis);
            a_axis_interval.min.total_cmp(&b_axis_interval.min)
        });
        let mid = range.start + (count as usize / 2);
        let left_node = BVH::build_helper(primitives, nodes, range.start..mid, depth + 1);
        let right_node = BVH::build_helper(primitives, nodes, mid..range.end, depth + 1);
        let node_index = nodes.len();
        let bounding_box = AABB::from_boxes(
            nodes[left_node].bounding_box,
            nodes[right_node].bounding_box,
        );

        let node = BVHNode {
            bounding_box,
            kind: BVHNodeKind::Inner {
                left: left_node as u32,
                right: right_node as u32,
            },
        };
        nodes.push(node);

        node_index
    }

    pub fn build(mut primitives: Vec<Primitive>) -> Self {
        let mut nodes: Vec<BVHNode> = Vec::new();
        let count = primitives.len();
        let index = BVH::build_helper(&mut primitives, &mut nodes, 0..count, 0);

        BVH {
            primitives,
            nodes,
            root: index,
        }
    }

    fn root_node(&self) -> &BVHNode {
        &self.nodes[self.root]
    }

    fn hit_leaf(&self, start: u32, count: u32, ray: &Ray, interval: Interval) -> Option<Hit> {
        let mut closest = interval.max;
        let mut hit_anything = None;

        for i in start..start + count {
            let primitive: &Primitive = &self.primitives[i as usize];

            if let Some(hit) = primitive.intersect(ray, Interval::new(interval.min, closest)) {
                closest = hit.t;
                hit_anything = Some(hit)
            }
        }
        return hit_anything;
    }
}

impl Intersect for BVH {
    fn intersect(&self, ray: &Ray, interval: Interval) -> Option<Hit> {
        if !self.bounding_box().hit(ray, interval) {
            return None;
        }

        let mut closest = interval.max;
        let mut hit_anything = None;
        // let mut stack = vec![self.root];
        let mut stack = [0; 64];
        let mut stack_top = 1;
        stack[0] = self.root;
        while stack_top > 0 {
            stack_top -= 1;
            // let node_index = stack.pop().unwrap();
            let node_index = stack[stack_top];
            let node = self.nodes[node_index];
            if !node
                .bounding_box
                .hit(ray, Interval::new(interval.min, closest))
            {
                continue;
            }

            match node.kind {
                BVHNodeKind::Leaf { start, count } => {
                    if let Some(hit) =
                        self.hit_leaf(start, count, ray, Interval::new(interval.min, closest))
                    {
                        closest = hit.t;
                        hit_anything = Some(hit);
                    }
                }
                BVHNodeKind::Inner { left, right } => {
                    stack[stack_top] = right as usize;
                    stack_top += 1;
                    stack[stack_top] = left as usize;
                    stack_top += 1;
                }
            }
        }
        hit_anything
    }

    fn bounding_box(&self) -> AABB {
        self.root_node().bounding_box
    }
}

#[cfg(test)]
mod tests {
    use core::f64;

    use crate::math::Vec3;

    use super::*;

    fn unit_sphere(center: Point3) -> Sphere {
        Sphere {
            center,
            radius: 1.0,
            material_id: 0,
        }
    }

    fn sphere(center: Point3, radius: f64) -> Sphere {
        Sphere {
            center,
            radius,
            material_id: 0,
        }
    }

    fn tri() -> Triangle {
        Triangle {
            p1: Point3::new(0.0, 0.0, 0.0),
            p2: Point3::new(1.0, 0.0, 0.0),
            p3: Point3::new(0.0, 1.0, 0.0),
            material_id: 0,
        }
    }

    #[test]
    fn sphere_direct_hit() {
        let sphere = unit_sphere(Point3::new(0.0, 0.0, -5.0));

        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 0.0, -1.0));

        let hit = sphere.intersect(&ray, Interval::new(0.0, 1000.0)).unwrap();

        assert!((hit.t - 4.0).abs() < 1e-6);
    }

    #[test]
    fn sphere_miss() {
        let sphere = unit_sphere(Point3::new(0.0, 0.0, -5.0));
        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 1.0, 0.0));

        let hit = sphere.intersect(&ray, Interval::new(0.0, 1000.0));

        assert!(hit.is_none());
    }

    #[test]
    fn sphere_tangent_hit() {
        let sphere = unit_sphere(Point3::new(0.0, 1.0, -5.0));
        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 0.0, -1.0));

        let hit = sphere.intersect(&ray, Interval::new(0.001, 1000.0));
        assert!(hit.is_some());

        let rec = hit.unwrap();

        assert!((rec.t - 5.0).abs() < 1e-6);
    }

    #[test]
    fn ray_origin_inside_sphere() {
        let sphere = unit_sphere(Point3::ORIGIN);
        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 0.0, 1.0));

        let hit = sphere.intersect(&ray, Interval::new(0.001, 1000.0));
        assert!(hit.is_some());

        let rec = hit.unwrap();

        assert!(rec.t > 0.0);
    }

    #[test]
    fn sphere_behind_ray() {
        let sphere = unit_sphere(Point3::new(0.0, 0.0, 5.0));
        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 0.0, -1.0));

        let hit = sphere.intersect(&ray, Interval::new(0.001, 1000.0));
        assert!(hit.is_none());
    }

    #[test]
    fn sphere_interval_rejection() {
        let sphere = unit_sphere(Point3::new(0.0, 0.0, -5.0));

        let ray = Ray::new(Point3::ORIGIN, Vec3::new(0.0, 0.0, -1.0));

        // Reject valid hit by shrinking interval
        let hit = sphere.intersect(&ray, Interval::new(0.001, 3.0));

        assert!(hit.is_none());
    }

    #[test]
    fn hit_center_front_face() {
        let sphere = unit_sphere(Point3::ORIGIN);

        let ray = Ray::new(Point3::new(0.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0));

        let hit = sphere
            .intersect(&ray, Interval::new(0.001, f64::INFINITY))
            .expect("Ray should hit sphere");

        // Hit point should be at z = -1
        assert!((hit.point - Point3::new(0.0, 0.0, -1.0)).length() < 1e-6);

        // Normal should point straight back toward camera
        assert!((hit.normal.0 - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-6);

        // Normal must be unit length
        assert!((hit.normal.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn hit_offset_point() {
        let sphere = unit_sphere(Point3::ORIGIN);

        let ray = Ray::new(
            Point3::new(0.0, 0.0, -3.0),
            Vec3::new(0.0, 0.2, 1.0).normalized(),
        );

        let hit = sphere
            .intersect(&ray, Interval::new(0.001, f64::INFINITY))
            .expect("Ray should hit sphere");

        // Normal must match radial direction
        let expected = (hit.point - Point3::ORIGIN).normalized();

        assert!((hit.normal.0 - expected).length() < 1e-6);
    }

    #[test]
    fn sphere_front_face() {
        let sphere = sphere(Point3::ORIGIN, 10.0);

        let ray_inner = Ray::new(Point3::ORIGIN, Vec3::new(1.0, 0.0, 0.0));

        let hit = sphere
            .intersect(&ray_inner, Interval::new(0.001, f64::INFINITY))
            .expect("Ray should hit the sphere");

        // A ray coming from within the sphere should have the correct `front_face` property
        assert!(!hit.front_face);

        let ray_outer = Ray::new(Point3::new(-15.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

        let hit = sphere
            .intersect(&ray_outer, Interval::new(0.001, f64::INFINITY))
            .expect("Ray should hit the sphere");

        // A ray comming from outside
        assert!(hit.front_face);
    }

    #[test]
    fn ray_hits_triangle_center() {
        let ray = Ray::new(Point3::new(0.25, 0.25, 1.0), Vec3::new(0.0, 0.0, -1.0));

        let triangle = tri();

        let hit = triangle.intersect(&ray, Interval::new(0.001, std::f64::INFINITY));

        assert!(hit.is_some());

        let rec = hit.unwrap();

        assert!((rec.t - 1.0).abs() < 1e-6);
        assert!((rec.point - Point3::new(0.25, 0.25, 0.0)).is_near_zero());
    }

    #[test]
    fn ray_misses_triangle() {
        let ray = Ray::new(Point3::new(1.1, 1.1, 1.0), Vec3::new(0.0, 0.0, -1.0));

        let triangle = tri();

        let hit = triangle.intersect(&ray, Interval::new(0.001, std::f64::INFINITY));

        assert!(hit.is_none())
    }

    #[test]
    fn parallel_ray_does_not_hit_triangle() {
        let ray = Ray::new(Point3::new(0.25, 0.25, 1.0), Vec3::new(1.0, 0.0, 0.0));

        let triangle = tri();

        let hit = triangle.intersect(&ray, Interval::new(0.001, std::f64::INFINITY));

        assert!(hit.is_none())
    }

    #[test]
    fn ray_hits_triangle_edge() {
        let ray = Ray::new(Point3::new(0.5, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));

        let triangle = tri();

        let hit = triangle.intersect(&ray, Interval::new(0.001, std::f64::INFINITY));

        assert!(hit.is_some())
    }

    #[test]
    fn ray_hits_triangle_vertex() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));

        let triangle = tri();

        let hit = triangle.intersect(&ray, Interval::new(0.001, std::f64::INFINITY));

        assert!(hit.is_some())
    }

    #[test]
    fn triangle_front_face() {
        let ray = Ray::new(Point3::new(0.25, 0.25, 1.0), Vec3::new(0.0, 0.0, -1.0));

        let triangle = tri();

        let rec = triangle
            .intersect(&ray, Interval::new(0.001, std::f64::INFINITY))
            .unwrap();

        assert!(rec.front_face);
    }

    #[test]
    fn triangle_back_face() {
        let ray = Ray::new(Point3::new(0.25, 0.25, -1.0), Vec3::new(0.0, 0.0, 1.0));

        let triangle = tri();

        let rec = triangle
            .intersect(&ray, Interval::new(0.001, std::f64::INFINITY))
            .unwrap();

        assert!(!rec.front_face);
    }

    #[test]
    fn aabb_default_constructor() {
        let r#box = AABB::new();
        assert!(r#box.x.min == f64::INFINITY);
        assert!(r#box.x.max == f64::NEG_INFINITY);
        assert!(r#box.y.min == f64::INFINITY);
        assert!(r#box.y.max == f64::NEG_INFINITY);
        assert!(r#box.z.min == f64::INFINITY);
        assert!(r#box.z.max == f64::NEG_INFINITY);
    }

    #[test]
    fn aabb_from_extrema() {
        let a = Point3::new(1.0, 2.0, 3.0);
        let b = Point3::new(4.0, 5.0, 6.0);
        let r#box = AABB::from_extrema(a, b);

        assert!(r#box.x.min == 1.0);
        assert!(r#box.x.max == 4.0);
        assert!(r#box.y.min == 2.0);
        assert!(r#box.y.max == 5.0);
        assert!(r#box.z.min == 3.0);
        assert!(r#box.z.max == 6.0);

        // Test with reversed points
        let r#box2 = AABB::from_extrema(b, a);
        assert!(r#box2.x.min == 1.0);
        assert!(r#box2.x.max == 4.0);
        assert!(r#box2.y.min == 2.0);
        assert!(r#box2.y.max == 5.0);
        assert!(r#box2.z.min == 3.0);
        assert!(r#box2.z.max == 6.0);
    }

    #[test]
    fn aabb_from_boxes() {
        let box1 = AABB::from_extrema(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
        let box2 = AABB::from_extrema(Point3::new(2.0, 2.0, 2.0), Point3::new(3.0, 3.0, 3.0));
        let combined = AABB::from_boxes(box1, box2);

        assert!(combined.x.min == 0.0);
        assert!(combined.x.max == 3.0);
        assert!(combined.y.min == 0.0);
        assert!(combined.y.max == 3.0);
        assert!(combined.z.min == 0.0);
        assert!(combined.z.max == 3.0);
    }

    #[test]
    fn aabb_hit_ray() {
        // Create a box from (-1,-1,-1) to (1,1,1)
        let r#box = AABB::from_extrema(Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, 1.0, 1.0));

        // Ray hitting the center of the front face
        let ray1 = Ray::new(Point3::new(0.0, 0.0, -2.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(r#box.hit(&ray1, Interval::new(0.0, 10.0)));

        // Ray missing the box (too high in y)
        let ray2 = Ray::new(Point3::new(0.0, 2.0, -2.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(!r#box.hit(&ray2, Interval::new(0.0, 10.0)));

        // Ray originating inside the box
        let ray3 = Ray::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(r#box.hit(&ray3, Interval::new(0.0, 10.0)));

        // Ray parallel to one axis but still hitting
        let ray4 = Ray::new(Point3::new(0.5, 0.5, -2.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(r#box.hit(&ray4, Interval::new(0.0, 10.0)));
    }

    #[test]
    fn aabb_axis_interval() {
        let r#box = AABB::from_extrema(Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, 5.0, 6.0));

        // X axis (index 0)
        assert!(r#box.axis_interval(0).min == 1.0);
        assert!(r#box.axis_interval(0).max == 4.0);

        // Y axis (index 1)
        assert!(r#box.axis_interval(1).min == 2.0);
        assert!(r#box.axis_interval(1).max == 5.0);

        // Z axis (index 2)
        assert!(r#box.axis_interval(2).min == 3.0);
        assert!(r#box.axis_interval(2).max == 6.0);

        // Default case (should be X axis)
        assert!(r#box.axis_interval(3).min == 1.0);
        assert!(r#box.axis_interval(3).max == 4.0);
    }
}
