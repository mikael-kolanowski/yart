# Yet Another Ray Tracer (YART)

A small, simple CPU ray tracer written in Rust that renders images to the PPM format.

Features
- Config-driven scene and renderer settings via TOML
- Outputs plain PPM images (viewable with common image tools)

Prerequisites
- Rust and Cargo installed

Build
```bash
cargo build --release
```

Run
```bash
# Run with a TOML config file
cargo run -- config.toml
```

Configuration
The renderer is configured using a TOML file.

Example `config.toml`:
```toml
[camera]
aspect_ratio = "16:9"
field_of_view = 90
position = "0, 0, 0"
look_at = "1, 1, 1"

[renderer]
samples_per_pixel = 100
max_bounces = 8

[image]
width = 400
output = "out.ppm"

[sky]
type = "linear-gradient"
from = "0, 0, 0"
to = "0.5, 0.7, 1"

[[materials]]
type = "lambertian"
name = "matte"

[[objects]]
type = "sphere"
position = "0, 0, -1"
radius = 0.5
material = "matte"
albedo = "0.1, 0.2, 0.5"
```

Output
- The program writes a PPM image to the path specified by the `image.output` field in the config (for example `out.ppm`).

# End-to-end testing
This project uses test images in `golden_images/` to detect regressions. When making fundamental changes, regenerate the golden images by setting the `UPDATE_GOLDENS` environment variable.
```bash
UPDATE_GOLDENS=1 cargo test
```