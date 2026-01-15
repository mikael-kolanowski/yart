use super::math::{Lerp, Vec3};

#[derive(Clone, Copy)]
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

    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r: r, g: g, b: b }
    }

    pub fn write(&self) {
        let ir = (255.999 * self.r) as i32;
        let ig = (255.999 * self.g) as i32;
        let ib = (255.999 * self.b) as i32;

        println!("{} {} {}", ir, ig, ib);
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
