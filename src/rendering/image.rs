use crate::color::Color;
use std::io::Write;

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: Vec::new(),
        }
    }

    pub fn add_pixel(&mut self, color: Color) {
        self.pixels.push(color);
    }

    pub fn write_ppm<W: Write>(&self, writer: &mut W) {
        let _ = writeln!(writer, "P3");
        let _ = writeln!(writer, "{} {}", self.width, self.height);
        let _ = writeln!(writer, "255"); // Max color value
        for color in &self.pixels {
            let _ = writeln!(writer, "{}", color.write());
        }
    }
}
