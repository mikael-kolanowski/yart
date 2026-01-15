pub trait Lerp<T> {
    fn lerp(start: T, end: T, t: f64) -> T;
}

pub mod ray;
pub mod geometry;
pub mod vector;

pub use ray::*;
pub use geometry::*;
pub use vector::*;
