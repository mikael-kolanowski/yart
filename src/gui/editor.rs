use log::error;
use std::path::PathBuf;

use eframe::egui;

use crate::config::ViewportConfig;
use crate::config::{Config, MaterialConfig, ObjectConfig, SkyConfig};
use crate::load_scene_from_config;
use crate::math::{Point3, Vec3};
use crate::rendering::sampler::RandomSampler;

use super::dialogs::{AddMaterialDialog, AddObjectDialog};
use super::property_editors;
use super::widgets;

pub struct Editor {
    config: Config,
    preview_texture: Option<egui::TextureHandle>,
    selected_object: Option<usize>,
    selected_material: Option<usize>,

    // Dialogs
    add_object_dialog: AddObjectDialog,
    add_material_dialog: AddMaterialDialog,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            config: default_config(),
            preview_texture: None,
            selected_object: None,
            selected_material: None,
            add_object_dialog: AddObjectDialog::new(),
            add_material_dialog: AddMaterialDialog::new(),
        }
    }

    pub fn with_config(config: &Config) -> Self {
        Self {
            config: config.clone(),
            ..Default::default()
        }
    }

    pub fn load_config(&mut self, config: Config) {
        self.config = config;
        self.preview_texture = None;
        self.selected_object = None;
        self.selected_material = None;
    }
}

fn default_config() -> Config {
    Config {
        camera: crate::config::CameraConfig {
            aspect_ratio: 16.0 / 9.0,
            field_of_view: 90,
            position: Point3::new(-1.0, 1.0, 1.0),
            look_at: Point3::new(0.0, 0.0, -1.0),
        },
        renderer: crate::config::RendererConfig {
            samples_per_pixel: 20,
            max_bounces: 10,
        },
        image: crate::config::ImageConfig {
            width: 400,
            output: PathBuf::from("output.ppm"),
        },
        materials: vec![MaterialConfig::Lambertian {
            name: "matte".to_string(),
            albedo: Vec3::new(0.5, 0.5, 0.5),
        }],
        objects: vec![ObjectConfig::Sphere {
            position: Vec3::new(0.0, 0.0, -1.0),
            radius: 1.0,
            material: "matte".to_string(),
        }],
        sky: SkyConfig::LinearGradient {
            from: Vec3::new(1.0, 1.0, 1.0),
            to: Vec3::new(0.5, 0.7, 1.0),
        },
        viewport: ViewportConfig::default(),
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load Scene").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("TOML files", &["toml"])
                            .pick_file()
                        {
                            if let Ok(config) = Config::from_path(path.as_path()) {
                                self.load_config(config);
                            }
                        }
                    }
                    if ui.button("Save Scene").clicked() {
                        if let Some(path) = rfd::FileDialog::new().save_file() {
                            self.config
                                .save_to_file(path.as_path())
                                .unwrap_or_else(|err| {
                                    error!("error while saving config: {err}");
                                });
                        }
                    }
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            widgets::panel_heading(ui, "Scene");

            ui.horizontal(|ui| {
                if ui.button("+ Add object").clicked() {
                    self.add_object_dialog.open(&self.config.materials);
                }

                if ui.button("+ Add material").clicked() {
                    self.add_material_dialog.open(&self.config.materials);
                }
            });

            egui::CollapsingHeader::new("Objects")
                .default_open(true)
                .show(ui, |ui| {
                    for (i, obj) in self.config.objects.iter().enumerate() {
                        let label = match obj {
                            ObjectConfig::Sphere {
                                position, radius, ..
                            } => {
                                format!(
                                    "Sphere ({:.1}, {:.1}, {:.1}) r={:.1}",
                                    position.x, position.y, position.z, radius
                                )
                            }
                            ObjectConfig::Triangle { .. } => "Triangle".to_string(),
                            ObjectConfig::Mesh { path, .. } => {
                                format!("Mesh ({})", path.display())
                            }
                        };

                        let is_selected = self.selected_object == Some(i);
                        if ui.selectable_label(is_selected, label).clicked() {
                            self.selected_object = Some(i);
                            self.selected_material = None;
                        }
                    }
                });

            egui::CollapsingHeader::new("Materials")
                .default_open(true)
                .show(ui, |ui| {
                    for (i, mat) in self.config.materials.iter().enumerate() {
                        let label = match mat {
                            MaterialConfig::Lambertian { name, .. } => {
                                format!("{} (Lambertian)", name)
                            }
                            MaterialConfig::Metal { name, .. } => {
                                format!("{} (Metal)", name)
                            }
                            MaterialConfig::NormalVisualization { name } => {
                                format!("{} (Normal)", name)
                            }
                        };

                        let is_selected = self.selected_material == Some(i);
                        if ui.selectable_label(is_selected, label).clicked() {
                            self.selected_material = Some(i);
                            self.selected_object = None;
                        }
                    }
                });

            ui.separator();
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            widgets::panel_heading(
                ui,
                format!(
                    "{} Properties",
                    if self.selected_material.is_some() {
                        "Material"
                    } else {
                        "Object"
                    }
                )
                .as_str(),
            );

            ui.vertical(|ui| {
                if let Some(obj_idx) = self.selected_object {
                    if let Some(obj) = self.config.objects.get_mut(obj_idx) {
                        property_editors::object(ui, obj, &self.config.materials);
                    }
                } else if let Some(mat_idx) = self.selected_material {
                    if let Some(mat) = self.config.materials.get_mut(mat_idx) {
                        property_editors::material(ui, mat);
                    }
                } else {
                    ui.label("Select an object or material to edit");
                }
            });

            ui.separator();

            widgets::panel_heading(ui, "Scene Settings");

            ui.vertical(|ui| {
                egui::CollapsingHeader::new("Camera")
                    .default_open(true)
                    .show(ui, |ui| {
                        property_editors::camera(ui, &mut self.config.camera);
                    });

                egui::CollapsingHeader::new("Renderer")
                    .default_open(true)
                    .show(ui, |ui| {
                        property_editors::renderer(ui, &mut self.config.renderer);
                    });

                egui::CollapsingHeader::new("Image")
                    .default_open(true)
                    .show(ui, |ui| {
                        property_editors::image(ui, &mut self.config.image);
                    });

                egui::CollapsingHeader::new("Sky")
                    .default_open(true)
                    .show(ui, |ui| {
                        property_editors::sky(ui, &mut self.config.sky);
                    });

                egui::CollapsingHeader::new("Viewport")
                    .default_open(true)
                    .show(ui, |ui| {
                        property_editors::viewport(ui, &mut self.config.viewport);
                    });
            });

            ui.separator();
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("main_grid")
                .num_columns(2)
                .spacing([10.0, 10.0])
                .min_col_width(200.0)
                .max_col_width(400.0)
                .show(ui, |ui| {
                    ui.set_min_size(egui::vec2(600.0, 400.0));

                    // Preview viewport
                    ui.vertical(|ui| {
                        let preview_size = egui::vec2(
                            (self.config.viewport.width as f32)
                                * (self.config.camera.aspect_ratio as f32),
                            self.config.viewport.width as f32,
                        );

                        let (rect, _response) =
                            ui.allocate_exact_size(preview_size, egui::Sense::click());

                        if let Some(ref texture) = self.preview_texture {
                            let image = egui::Image::new(texture);
                            image.max_size(rect.size()).paint_at(ui, rect);
                        } else {
                            ui.painter()
                                .rect_filled(rect, 0.0, egui::Color32::DARK_GRAY);
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "Click 'Render Preview' to render",
                                egui::FontId::default(),
                                egui::Color32::WHITE,
                            );
                        }
                    });

                    ui.end_row();
                });

            // Bottom buttons
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Render Preview").clicked() {
                    if let Some(color_image) = render_preview(&self.config) {
                        self.preview_texture = Some(ctx.load_texture(
                            "preview",
                            color_image,
                            egui::TextureOptions::NEAREST,
                        ));
                    }
                }
            });
        });

        // Object Dialog
        if let Some(obj) = self.add_object_dialog.show(ctx, &self.config.materials) {
            self.config.objects.push(obj);
        }

        // Material Dialog
        if let Some(mat) = self.add_material_dialog.show(ctx, &self.config.materials) {
            self.config.materials.push(mat);
        }
    }
}

fn render_preview(config: &Config) -> Option<egui::ColorImage> {
    let viewport = &config.viewport;

    let mut rng = rand::rng();
    let mut sampler = RandomSampler::new(&mut rng);

    let preview_config = Config {
        camera: crate::config::CameraConfig {
            aspect_ratio: config.camera.aspect_ratio,
            field_of_view: config.camera.field_of_view,
            position: config.camera.position,
            look_at: config.camera.look_at,
        },
        renderer: crate::config::RendererConfig {
            samples_per_pixel: viewport.samples_per_pixel,
            max_bounces: viewport.max_bounces,
        },
        image: crate::config::ImageConfig {
            width: viewport.width,
            output: PathBuf::from("/dev/null"),
        },
        materials: config.materials.clone(),
        objects: config.objects.clone(),
        sky: config.sky.clone(),
        viewport: ViewportConfig::default(),
    };

    let (camera, world, renderer) = load_scene_from_config(&preview_config, &PathBuf::from("."));

    let image = renderer.render(&world, &camera, &mut sampler, false);

    let width = image.width as usize;
    let height = image.height as usize;

    let mut rgba_data = Vec::with_capacity(width * height * 4);

    for y in 0..height {
        for x in 0..width {
            let pixel = image.pixels[y * width + x];
            rgba_data.push((pixel.r * 255.0) as u8);
            rgba_data.push((pixel.g * 255.0) as u8);
            rgba_data.push((pixel.b * 255.0) as u8);
            rgba_data.push(255);
        }
    }

    Some(egui::ColorImage::from_rgba_unmultiplied(
        [width, height],
        &rgba_data,
    ))
}
