use crate::{color::Color, math::interval::Interval};
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub enum Error {
    PPMParseError,
}

#[derive(Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
}

fn linear_to_gamma(color: &Color) -> Color {
    color.map(|component| {
        if component > 0.0 {
            component.sqrt()
        } else {
            0.0
        }
    })
}

fn color_to_ppm(color: &Color) -> String {
    let c = linear_to_gamma(color); //let c = self;
    // Translate the [0, 1] component values to the range [0, 255]
    let intensity = Interval::new(0.0, 0.999);
    let ir = (256.0 * intensity.clamp(c.r)) as i32;
    let ig = (256.0 * intensity.clamp(c.g)) as i32;
    let ib = (256.0 * intensity.clamp(c.b)) as i32;

    return format!("{} {} {}", ir, ig, ib);
}

fn gamma_to_linear(color: &Color) -> Color {
    color.map(|component| component * component)
}

fn color_from_ppm(s: &str) -> Option<Color> {
    let parts: Vec<&str> = s.split(" ").collect();
    let r: u32 = parts[0].parse().ok()?;
    let g: u32 = parts[1].parse().ok()?;
    let b: u32 = parts[2].parse().ok()?;

    let color = Color::new(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    Some(gamma_to_linear(&color))
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: Vec::with_capacity(width as usize * height as usize),
        }
    }

    pub fn read_from_ppm<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut contents = String::new();
        reader.read_to_string(&mut contents).unwrap();

        let mut lines = contents.lines();

        let (width, height) = lines
            .nth(1)
            .ok_or(Error::PPMParseError)?
            .split_once(" ")
            .ok_or(Error::PPMParseError)?;

        let width: u32 = width.parse().map_err(|_err| Error::PPMParseError)?;
        let height: u32 = height.parse().map_err(|_err| Error::PPMParseError)?;

        // Skip max color value
        let _ = lines.next();

        let mut pixels: Vec<Color> = Vec::with_capacity(width as usize * height as usize);

        for line in lines {
            if let Some(color) = color_from_ppm(line) {
                pixels.push(color);
            } else {
                return Err(Error::PPMParseError);
            }
        }

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn add_pixel(&mut self, color: Color) {
        self.pixels.push(color);
    }

    pub fn write_ppm<W: Write>(&self, writer: &mut W) {
        let _ = writeln!(writer, "P3");
        let _ = writeln!(writer, "{} {}", self.width, self.height);
        let _ = writeln!(writer, "255"); // Max color value
        for color in &self.pixels {
            let _ = writeln!(writer, "{}", color_to_ppm(color));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, iter::zip};

    use crate::math::Vec3;

    use super::*;

    fn assert_images_are_close(expected: &Image, actual: &Image) {
        assert_eq!(actual.width, expected.width);
        assert_eq!(actual.height, expected.height);

        let threshold = 1e-4;
        let total_squared_error: f64 = zip(actual.pixels.iter(), expected.pixels.iter())
            .map(|(&a, &e)| a - e)
            .map(|delta| Vec3::new(delta.r, delta.g, delta.b).length_squared())
            .sum();

        let mse = total_squared_error / (actual.width as f64 * actual.height as f64);
        assert!(
            mse < threshold,
            "expected MSE {} to be less than threshold {}",
            mse,
            threshold
        );
    }

    #[test]
    fn write_and_read_simple() {
        let mut image = Image::new(3, 2);
        image.add_pixel(Color::new(1.0, 0.0, 0.0));
        image.add_pixel(Color::new(0.0, 1.0, 0.0));
        image.add_pixel(Color::new(0.0, 0.0, 1.0));

        image.add_pixel(Color::new(1.0, 1.0, 0.0));
        image.add_pixel(Color::new(1.0, 1.0, 1.0));
        image.add_pixel(Color::new(0.0, 0.0, 0.0));

        let mut buffer = Cursor::new(Vec::new());

        image.write_ppm(&mut buffer);
        buffer.set_position(0);

        let read_image = Image::read_from_ppm(&mut buffer).expect("Unable to read image");

        assert_images_are_close(&read_image, &image);
    }

    #[test]
    fn write_and_read_gradient() {
        let width = 64;
        let height = 64;

        let mut image = Image::new(width, height);

        for j in 0..height {
            for i in 0..width {
                let r = i as f64 / (width - 1) as f64;
                let g = j as f64 / (height - 1) as f64;
                let b = 0.0;

                image.add_pixel(Color::new(r, g, b));
            }
        }

        let mut buffer = Cursor::new(Vec::new());

        image.write_ppm(&mut buffer);
        buffer.set_position(0);

        let read_image = Image::read_from_ppm(&mut buffer).expect("Unable to read image");

        assert_images_are_close(&read_image, &image);
    }
}
