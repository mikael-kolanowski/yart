mod math;

struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    fn write(&self) {
        let ir = (255.999 * self.r) as i32;
        let ig = (255.999 * self.g) as i32;
        let ib = (255.999 * self.b) as i32;

        println!("{} {} {}", ir, ig, ib);
    }
}

fn main() {
    // PPM
    let width = 256;
    let height = 256;

    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = {
        let w = (image_width as f64 * aspect_ratio) as i32;
        if w < 1 {
            1
        } else {
            w
        }
    };
    
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width as f64) / (image_height as f64);

    println!("P3");
    println!("{} {}", width, height);
    println!("255"); // Max color component

    for j in 0..height {
        for i in 0..width {
            let color = Color {
                r: (i as f64) / (width - 1) as f64,
                g: (j as f64) / (height - 1) as f64,
                b: 0.0,
            };
            color.write();
        }
    }
}
