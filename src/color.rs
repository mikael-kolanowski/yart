use super::math::{Lerp, Vec3};
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };

    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };

    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r: r, g: g, b: b }
    }

    pub fn map(&self, f: fn(f64) -> f64) -> Self {
        Self {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }
}

impl From<Vec3> for Color {
    fn from(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl Lerp<Color> for Color {
    fn lerp(start: Color, end: Color, t: f64) -> Self {
        let va = Vec3::new(start.r, start.g, start.b);
        let vb = Vec3::new(end.r, end.g, end.b);
        let lerped = Vec3::lerp(va, vb, t);
        Color::new(lerped.x, lerped.y, lerped.z)
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
        }
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
        }
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}
