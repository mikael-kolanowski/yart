use std::path::Path;
use std::path::PathBuf;

use eframe::egui;
use log::error;

use crate::config::{Config, MaterialConfig, ObjectConfig};
use crate::load_scene_from_config;
use crate::rendering::sampler::RandomSampler;

use super::dialogs::{AddMaterialDialog, AddObjectDialog, HelpDialog};
use super::property_editors;
use super::shortcuts::Shortcuts;
use super::utils;
use super::widgets;

pub struct ViewportRendererConfig {
    pub samples_per_pixel: u32,
    pub max_bounces: u32,
}

pub struct Editor {
    config: Config,
    viewport_renderer: ViewportRendererConfig,
    preview_texture: Option<egui::TextureHandle>,
    selected_object: Option<usize>,
    selected_material: Option<usize>,

    // Dialogs
    add_object_dialog: AddObjectDialog,
    add_material_dialog: AddMaterialDialog,
    help_dialog: HelpDialog,

    // Shortcuts
    shortcuts: Shortcuts,

    asset_base_path: PathBuf,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            config: utils::default_config(),
            viewport_renderer: ViewportRendererConfig {
                samples_per_pixel: 10,
                max_bounces: 10,
            },
            preview_texture: None,
            selected_object: None,
            selected_material: None,
            add_object_dialog: AddObjectDialog::new(),
            add_material_dialog: AddMaterialDialog::new(),
            help_dialog: HelpDialog::new(),
            shortcuts: Shortcuts::new(),
            asset_base_path: PathBuf::new(),
        }
    }

    pub fn with_config(config: &Config, asset_base_path: &Path) -> Self {
        Self {
            config: config.clone(),
            asset_base_path: PathBuf::from(asset_base_path),
            ..Default::default()
        }
    }

    pub fn load_config(&mut self, config: Config) {
        self.config = config;
        self.preview_texture = None;
        self.selected_object = None;
        self.selected_material = None;
    }

    fn render_preview(&self, available_size: egui::Rect) -> Option<egui::ColorImage> {
        let mut rng = rand::rng();
        let mut sampler = RandomSampler::new(&mut rng);

        let preview_width = available_size.max.x;
        let preview_height = available_size.max.y;
        let preview_aspect_ratio = (preview_width / preview_height) as f64;

        let preview_config = Config {
            camera: crate::CameraConfig {
                aspect_ratio: preview_aspect_ratio,
                ..self.config.camera.clone()
            },
            renderer: crate::RendererConfig {
                max_bounces: self.viewport_renderer.max_bounces,
                samples_per_pixel: self.viewport_renderer.samples_per_pixel,
            },
            ..self.config.clone()
        };

        let (camera, world, renderer) =
            load_scene_from_config(&preview_config, &self.asset_base_path);

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
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui_top_menu(ctx);
        self.ui_left_panel(ctx);
        self.ui_right_panel(ctx);
        self.ui_central_panel(ctx);
        self.ui_dialogs(ctx);
    }
}

impl Editor {
    fn ui_top_menu(&mut self, ctx: &egui::Context) {
        // Check for help shortcut
        if self.shortcuts.is_pressed(ctx, &self.shortcuts.show_help) {
            self.help_dialog.open();
        }

        if self.shortcuts.is_pressed(ctx, &self.shortcuts.load_scene) {
            self.handle_load_scene();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load Scene").clicked() {
                        self.handle_load_scene();
                    }
                    if ui.button("Save Scene").clicked() {
                        self.handle_save_scene();
                    }
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Keyboard Shortcuts").clicked() {
                        self.help_dialog.open();
                        ui.close();
                    }
                });
            });
        });
    }

    fn ui_left_panel(&mut self, ctx: &egui::Context) {
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
                            MaterialConfig::Dielectric { name, .. } => {
                                format!("{} (Dielectric)", name)
                            }
                            MaterialConfig::DiffuseLight { name, .. } => {
                                format!("{} (Diffuse Light)", name)
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
    }

    fn ui_right_panel(&mut self, ctx: &egui::Context) {
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
                        property_editors::viewport(ui, &mut self.viewport_renderer);
                    });
            });

            ui.separator();
        });
    }

    fn preview_size(&mut self, ui: &mut egui::Ui) -> egui::Rect {
        egui::Rect::from_two_pos(
            egui::Pos2::new(0.0, 0.0),
            egui::Pos2::new(ui.available_width(), ui.available_height()),
        )
    }

    fn render_preview_to_texture(&mut self, ctx: &egui::Context, available_size: egui::Rect) {
        if let Some(color_image) = self.render_preview(available_size) {
            self.preview_texture =
                Some(ctx.load_texture("preview", color_image, egui::TextureOptions::NEAREST));
        }
    }

    fn ui_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Check for keyboard shortcuts
            if self
                .shortcuts
                .is_pressed(ctx, &self.shortcuts.render_preview)
            {
                let preview_size = self.preview_size(ui);
                self.render_preview_to_texture(ctx, preview_size);
            }

            // Preview viewport
            let preview_size = self.preview_size(ui);
            ui.vertical(|ui| {
                let (rect, _response) =
                    ui.allocate_exact_size(preview_size.max.to_vec2(), egui::Sense::click());

                if let Some(ref texture) = self.preview_texture {
                    let image = egui::Image::new(texture);
                    image.paint_at(ui, rect);
                } else {
                    ui.painter()
                        .rect_filled(rect, 0.0, egui::Color32::DARK_GRAY);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Click 'Render Preview' (or press Ctrl+R) to render",
                        egui::FontId::default(),
                        egui::Color32::WHITE,
                    );
                }
            });

            ui.end_row();
        });
    }

    fn ui_dialogs(&mut self, ctx: &egui::Context) {
        // Object Dialog
        if let Some(obj) = self.add_object_dialog.show(ctx, &self.config.materials) {
            self.config.objects.push(obj);
        }

        // Material Dialog
        if let Some(mat) = self.add_material_dialog.show(ctx, &self.config.materials) {
            self.config.materials.push(mat);
        }

        // Help Dialog
        self.help_dialog.show(ctx, &self.shortcuts);
    }

    fn handle_load_scene(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("TOML files", &["toml"])
            .pick_file()
        {
            if let Ok(config) = Config::from_path(path.as_path()) {
                self.load_config(config);
            }
        }
    }

    fn handle_save_scene(&mut self) {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            self.config
                .save_to_file(path.as_path())
                .unwrap_or_else(|err| {
                    error!("error while saving config: {err}");
                });
        }
    }
}
