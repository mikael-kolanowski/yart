use super::math::{Lerp, Vec3, interval::Interval};
use std::ops::{Add, Mul};

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

    fn map(&self, f: fn(f64) -> f64) -> Self {
        Self {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }

    fn linear_to_gamma(&self) -> Self {
        self.map(|component| {
            if component > 0.0 {
                component.sqrt()
            } else {
                0.0
            }
        })
    }

    pub fn write(&self) -> String {
        let c = self.linear_to_gamma();

        // Translate the [0, 1] component values to the range [0, 255]
        let intensity = Interval::new(0.0, 0.999);
        let ir = (256.0 * intensity.clamp(c.r)) as i32;
        let ig = (256.0 * intensity.clamp(c.g)) as i32;
        let ib = (256.0 * intensity.clamp(c.b)) as i32;

        return format!("{} {} {}", ir, ig, ib);
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
