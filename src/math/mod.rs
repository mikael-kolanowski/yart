pub trait Lerp<T> {
    fn lerp(start: T, end: T, t: f64) -> T;
}

pub mod geometry;
pub mod interval;
pub mod ray;
pub mod vector;

pub use geometry::*;
pub use ray::*;
pub use vector::*;
