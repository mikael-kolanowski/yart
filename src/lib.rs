pub mod color;
pub mod config;
pub mod gui;
pub mod math;
pub mod mesh;
pub mod progressbar;
pub mod rendering;
pub mod world;

use std::path::Path;

pub use crate::config::*;
pub use crate::rendering::*;
pub use crate::world::World;

pub fn load_scene_from_config(
    config: &Config,
    asset_base_path: &Path,
) -> (Camera, World, Renderer) {
    let aspect_ratio = config.camera.aspect_ratio.0 as f64 / config.camera.aspect_ratio.1 as f64;
    let camera = Camera::new(
        aspect_ratio,
        config.image.width,
        config.camera.field_of_view,
        config.camera.position,
        config.camera.look_at,
    );

    let world = World::from_config(config, asset_base_path);
    let renderer = Renderer::new(
        config.renderer.samples_per_pixel,
        config.renderer.max_bounces,
    );

    (camera, world, renderer)
}
