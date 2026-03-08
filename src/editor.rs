use std::path::PathBuf;

use eframe::egui::{self, Context};

use crate::config::ViewportConfig;
use crate::config::{Config, MaterialConfig, ObjectConfig, SkyConfig};
use crate::load_scene_from_config;
use crate::math::{Point3, Vec3};
use crate::rendering::sampler::RandomSampler;

pub struct Editor {
    config: Config,
    preview_texture: Option<egui::TextureHandle>,
    selected_object: Option<usize>,
    selected_material: Option<usize>,

    // Dialog state
    add_object_dialog_open: bool,
    add_material_dialog_open: bool,
    pending_object: Option<ObjectConfig>,
    pending_object_error: Option<String>,
    pending_material: Option<MaterialConfig>,
    pending_material_error: Option<String>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            config: default_config(),
            preview_texture: None,
            selected_object: None,
            selected_material: None,
            add_object_dialog_open: false,
            add_material_dialog_open: false,
            pending_object: None,
            pending_object_error: None,
            pending_material: None,
            pending_material_error: None,
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

    fn render_add_object_dialog(&mut self, ctx: &Context) {
        let mut cancel_clicked = false;
        let mut add_clicked = false;
        let default_material = self
            .config
            .materials
            .first()
            .map(|m| m.name().to_string())
            .unwrap_or_default();

        egui::Window::new("Add Object")
            .open(&mut self.add_object_dialog_open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(ref mut obj) = self.pending_object {
                    object_type_selector(ui, obj, &default_material);
                    edit_object_properties(ui, obj, &self.config.materials);
                }

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        cancel_clicked = true;
                    }
                    if ui.button("Add").clicked() {
                        add_clicked = true;
                    }
                });

                if let Some(ref error) = self.pending_object_error {
                    ui.colored_label(egui::Color32::RED, error);
                }
            });

        if cancel_clicked {
            self.add_object_dialog_open = false;
        }
        if add_clicked {
            if let Some(ref obj) = self.pending_object {
                if let Err(e) = validate_object(obj, &self.config.materials) {
                    self.pending_object_error = Some(e);
                } else {
                    self.config.objects.push(obj.clone());
                    self.add_object_dialog_open = false;
                }
            }
        }
    }

    fn render_add_material_dialog(&mut self, ctx: &Context) {
        let mut cancel_clicked = false;
        let mut add_clicked = false;

        egui::Window::new("Add Material")
            .open(&mut self.add_material_dialog_open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(ref mut mat) = self.pending_material {
                    material_type_selector(ui, mat, &self.config.materials);
                    edit_material_properties(ui, mat);
                }

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        cancel_clicked = true;
                    }
                    if ui.button("Add").clicked() {
                        add_clicked = true;
                    }
                });

                if let Some(ref error) = self.pending_material_error {
                    ui.colored_label(egui::Color32::RED, error);
                }
            });

        if cancel_clicked {
            self.add_material_dialog_open = false;
        }
        if add_clicked {
            if let Some(ref mat) = self.pending_material {
                if let Err(e) = validate_material(mat, &self.config.materials) {
                    self.pending_material_error = Some(e);
                } else {
                    self.config.materials.push(mat.clone());
                    self.add_material_dialog_open = false;
                }
            }
        }
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

fn panel_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).strong().heading().size(14.0));
}

fn vector_input(ui: &mut egui::Ui, vector: &mut Vec3) {
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut vector.x).speed(0.1))
            .on_hover_text("x");
        ui.add(egui::DragValue::new(&mut vector.y).speed(0.1))
            .on_hover_text("y");
        ui.add(egui::DragValue::new(&mut vector.z).speed(0.1))
            .on_hover_text("z");
    });
}

fn color_input(ui: &mut egui::Ui, color: &mut Vec3) {
    let mut as_f32_array = [color.x as f32, color.y as f32, color.z as f32];

    ui.horizontal(|ui| {
        vector_input(ui, color);
        egui::color_picker::color_edit_button_rgb(ui, &mut as_f32_array);
    });

    *color = Vec3::new(
        as_f32_array[0] as f64,
        as_f32_array[1] as f64,
        as_f32_array[2] as f64,
    );
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
                        println!("TODO: export config");
                    }
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            panel_heading(ui, "Scene");

            ui.horizontal(|ui| {
                if ui.button("+ Add object").clicked() {
                    self.pending_object = Some(ObjectConfig::Sphere {
                        position: Vec3::new(0.0, 0.0, -1.0),
                        radius: 1.0,
                        material: self
                            .config
                            .materials
                            .first()
                            .map(|m| m.name().to_string())
                            .unwrap_or_default(),
                    });
                    self.pending_object_error = None;
                    self.add_object_dialog_open = true;
                }

                if ui.button("+ Add material").clicked() {
                    self.pending_material = Some(MaterialConfig::Lambertian {
                        name: new_material_name("lambertian", &self.config.materials),
                        albedo: Vec3::new(0.5, 0.5, 0.5),
                    });
                    self.pending_material_error = None;
                    self.add_material_dialog_open = true;
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
            panel_heading(
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
                        edit_object_properties(ui, obj, &self.config.materials);
                    }
                } else if let Some(mat_idx) = self.selected_material {
                    if let Some(mat) = self.config.materials.get_mut(mat_idx) {
                        edit_material_properties(ui, mat);
                    }
                } else {
                    ui.label("Select an object or material to edit");
                }
            });

            ui.separator();

            panel_heading(ui, "Scene Settings");

            ui.vertical(|ui| {
                egui::CollapsingHeader::new("Camera")
                    .default_open(true)
                    .show(ui, |ui| {
                        edit_camera_config(ui, &mut self.config.camera);
                    });

                egui::CollapsingHeader::new("Renderer")
                    .default_open(true)
                    .show(ui, |ui| {
                        edit_renderer_config(ui, &mut self.config.renderer);
                    });

                egui::CollapsingHeader::new("Image")
                    .default_open(true)
                    .show(ui, |ui| {
                        edit_image_config(ui, &mut self.config.image);
                    });

                egui::CollapsingHeader::new("Sky")
                    .default_open(true)
                    .show(ui, |ui| {
                        edit_sky_config(ui, &mut self.config.sky);
                    });

                egui::CollapsingHeader::new("Viewport")
                    .default_open(true)
                    .show(ui, |ui| {
                        edit_viewport_config(ui, &mut self.config.viewport);
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
        if self.add_object_dialog_open {
            self.render_add_object_dialog(ctx);
        }

        // Material Dialog
        if self.add_material_dialog_open {
            self.render_add_material_dialog(ctx);
        }
    }
}

fn object_type_selector(ui: &mut egui::Ui, obj: &mut ObjectConfig, default_material: &str) {
    let current_type = match obj {
        ObjectConfig::Sphere { .. } => "sphere",
        ObjectConfig::Triangle { .. } => "triangle",
        ObjectConfig::Mesh { .. } => "mesh",
    };
    let mut obj_type = current_type.to_string();

    ui.horizontal(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt("object_type_dialog")
            .selected_text(&obj_type)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut obj_type, "sphere".to_string(), "Sphere");
                ui.selectable_value(&mut obj_type, "triangle".to_string(), "Triangle");
                ui.selectable_value(&mut obj_type, "mesh".to_string(), "Mesh");
            });
    });

    if obj_type != current_type {
        *obj = match obj_type.as_str() {
            "sphere" => ObjectConfig::Sphere {
                position: Vec3::new(0.0, 0.0, -1.0),
                radius: 1.0,
                material: default_material.to_string(),
            },
            "mesh" => ObjectConfig::Mesh {
                path: PathBuf::new(),
                material: default_material.to_string(),
            },
            _ => obj.clone(),
        };
    }
}

fn material_type_selector(
    ui: &mut egui::Ui,
    mat: &mut MaterialConfig,
    existing: &[MaterialConfig],
) {
    let current_type = material_label(&mat);

    let mut mat_type = current_type.to_string();

    ui.horizontal(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt("material_type_dialog")
            .selected_text(&mat_type)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut mat_type, "lambertian".to_string(), "Lambertian");
                ui.selectable_value(&mut mat_type, "metal".to_string(), "Metal");
                ui.selectable_value(
                    &mut mat_type,
                    "normal_vis".to_string(),
                    "Normal Visualization",
                );
            });
    });

    if mat_type != current_type {
        let default_name = new_material_name(&mat_type, existing);
        *mat = match mat_type.as_str() {
            "lambertian" => MaterialConfig::Lambertian {
                name: default_name,
                albedo: Vec3::new(0.5, 0.5, 0.5),
            },
            "metal" => MaterialConfig::Metal {
                name: default_name,
                albedo: Vec3::new(0.5, 0.5, 0.5),
                fuzz: 0.3,
            },
            "normal_vis" => MaterialConfig::NormalVisualization { name: default_name },
            _ => mat.clone(),
        };
    }
}

fn new_material_name(
    selected_material_label: &str,
    existing_materials: &[MaterialConfig],
) -> String {
    let n = existing_materials
        .iter()
        .filter(|mat| material_label(mat).eq(selected_material_label))
        .count();
    format!("{selected_material_label}_{n}")
}

fn validate_object(obj: &ObjectConfig, materials: &[MaterialConfig]) -> Result<(), String> {
    let material_name = match obj {
        ObjectConfig::Sphere { material, .. } => material,
        ObjectConfig::Triangle { material, .. } => material,
        ObjectConfig::Mesh { material, .. } => material,
    };

    let material_exists = materials.iter().any(|m| m.name() == *material_name);
    if !material_exists {
        return Err("Material not found".to_string());
    }

    if let ObjectConfig::Mesh { path, .. } = obj {
        if path.as_os_str().is_empty() {
            return Err("Mesh path is required".to_string());
        }
    }

    Ok(())
}

fn validate_material(mat: &MaterialConfig, existing: &[MaterialConfig]) -> Result<(), String> {
    let name = mat.name();

    if name.is_empty() {
        return Err("Name is required".to_string());
    }

    let name_unique = !existing.iter().any(|m| m.name() == name);
    if !name_unique {
        return Err("Name must be unique".to_string());
    }

    Ok(())
}

fn edit_object_properties(ui: &mut egui::Ui, obj: &mut ObjectConfig, materials: &[MaterialConfig]) {
    egui::Grid::new("object_properties_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| match obj {
            ObjectConfig::Sphere {
                position,
                radius,
                material,
            } => {
                ui.label("Type:");
                ui.label("Sphere");
                ui.end_row();

                ui.label("Position:");
                ui.horizontal(|ui| {
                    vector_input(ui, position);
                });
                ui.end_row();

                ui.label("Radius:");
                ui.add(egui::DragValue::new(radius).speed(0.1));
                ui.end_row();

                ui.label("Material:");
                egui::ComboBox::from_id_salt("material_select")
                    .selected_text(material.clone())
                    .show_ui(ui, |ui| {
                        for mat in materials {
                            ui.selectable_value(material, mat.name().to_string(), mat.name());
                        }
                    });
                ui.end_row();
            }
            ObjectConfig::Mesh { path, material } => {
                let mut display_path = path.to_string_lossy().to_string();
                ui.label("Type:");
                ui.label("Mesh");
                ui.end_row();

                ui.label("Path:");
                ui.text_edit_singleline(&mut display_path);
                if ui.button("Browse").clicked() {
                    if let Some(selected_path) = rfd::FileDialog::new()
                        .add_filter("OBJ files", &["obj"])
                        .pick_file()
                    {
                        display_path = selected_path.display().to_string();
                    }
                }
                *path = PathBuf::from(display_path);
                ui.end_row();

                ui.label("Material:");
                egui::ComboBox::from_id_salt("material_select")
                    .selected_text(material.clone())
                    .show_ui(ui, |ui| {
                        for mat in materials {
                            ui.selectable_value(material, mat.name().to_string(), mat.name());
                        }
                    });
                ui.end_row();
            }
            _ => {}
        });
}

fn edit_material_properties(ui: &mut egui::Ui, mat: &mut MaterialConfig) {
    egui::Grid::new("object_properties_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| match mat {
            MaterialConfig::Lambertian { name, albedo } => {
                ui.label("Type:");
                ui.label("Lambertian");
                ui.end_row();

                ui.label("Name:");
                ui.text_edit_singleline(name);
                ui.end_row();

                ui.label("Albedo:");
                color_input(ui, albedo);
            }
            MaterialConfig::Metal { name, albedo, fuzz } => {
                ui.label("Type:");
                ui.label("Metal");
                ui.end_row();

                ui.label("Name:");
                ui.text_edit_singleline(name);
                ui.end_row();

                ui.label("Albedo:");
                color_input(ui, albedo);
                ui.end_row();

                ui.label("Fuzz:");
                ui.add(egui::Slider::new(fuzz, 0.0..=1.0));
            }
            MaterialConfig::NormalVisualization { name } => {
                ui.label("Type:");
                ui.label("Normal visualizer");
                ui.end_row();

                ui.label("Name:");
                ui.text_edit_singleline(name);
                ui.end_row();
            }
        });
}

fn edit_camera_config(ui: &mut egui::Ui, camera: &mut crate::config::CameraConfig) {
    egui::Grid::new("camera_config_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Aspect Ratio:");
            ui.add(egui::DragValue::new(&mut camera.aspect_ratio).speed(0.1));
            ui.end_row();

            ui.label("Field of View:");
            ui.add(egui::DragValue::new(&mut camera.field_of_view).speed(1.0));
            ui.end_row();

            ui.label("Position:");
            ui.horizontal(|ui| {
                vector_input(ui, &mut camera.position.0);
            });
            ui.end_row();

            ui.label("Look At:");
            ui.horizontal(|ui| {
                vector_input(ui, &mut camera.look_at.0);
            });
            ui.end_row();
        });
}

fn edit_renderer_config(ui: &mut egui::Ui, renderer: &mut crate::config::RendererConfig) {
    egui::Grid::new("renderer_config_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Samples per Pixel:");
            ui.add(egui::DragValue::new(&mut renderer.samples_per_pixel).speed(1.0));
            ui.end_row();

            ui.label("Max Bounces:");
            ui.add(egui::DragValue::new(&mut renderer.max_bounces).speed(1.0));
            ui.end_row();
        });
}

fn edit_image_config(ui: &mut egui::Ui, image: &mut crate::config::ImageConfig) {
    egui::Grid::new("image_config_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Width:");
            ui.add(egui::DragValue::new(&mut image.width).speed(10.0));
            ui.end_row();

            ui.label("Output:");
            let mut output_str = image.output.to_string_lossy().to_string();
            ui.text_edit_singleline(&mut output_str);
            image.output = PathBuf::from(output_str);
            ui.end_row();
        });
}

fn edit_sky_config(ui: &mut egui::Ui, sky: &mut crate::config::SkyConfig) {
    let current_type = match sky {
        crate::config::SkyConfig::LinearGradient { .. } => "linear-gradient",
        crate::config::SkyConfig::Solid { .. } => "solid",
    };
    let mut sky_type = current_type.to_string();

    egui::Grid::new("sky_config_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Type:");
            egui::ComboBox::from_id_salt("sky_type")
                .selected_text(&sky_type)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut sky_type,
                        "linear-gradient".to_string(),
                        "Linear Gradient",
                    );
                    ui.selectable_value(&mut sky_type, "solid".to_string(), "Solid Color");
                });
            ui.end_row();

            match sky {
                crate::config::SkyConfig::LinearGradient { from, to } => {
                    ui.label("From:");
                    color_input(ui, from);
                    ui.end_row();

                    ui.label("To:");
                    color_input(ui, to);
                    ui.end_row();
                }
                crate::config::SkyConfig::Solid { color } => {
                    ui.label("Color:");
                    color_input(ui, color);
                    ui.end_row();
                }
            }
        });

    if sky_type != current_type {
        *sky = match sky_type.as_str() {
            "linear-gradient" => crate::config::SkyConfig::LinearGradient {
                from: Vec3::new(1.0, 1.0, 1.0),
                to: Vec3::new(0.5, 0.7, 1.0),
            },
            "solid" => crate::config::SkyConfig::Solid {
                color: Vec3::new(0.5, 0.7, 1.0),
            },
            _ => sky.clone(),
        };
    }
}

fn edit_viewport_config(ui: &mut egui::Ui, viewport: &mut ViewportConfig) {
    egui::Grid::new("viewport_config_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Preview Width:");
            ui.add(egui::DragValue::new(&mut viewport.width).speed(10.0));
            ui.end_row();

            ui.label("Preview Samples:");
            ui.add(egui::DragValue::new(&mut viewport.samples_per_pixel).speed(1.0));
            ui.end_row();

            ui.label("Preview Bounces:");
            ui.add(egui::DragValue::new(&mut viewport.max_bounces).speed(1.0));
            ui.end_row();
        });
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

fn material_label(mat: &MaterialConfig) -> String {
    match mat {
        MaterialConfig::Lambertian { .. } => "lambertian".into(),
        MaterialConfig::Metal { .. } => "metal".into(),
        MaterialConfig::NormalVisualization { .. } => "normal_vis".into(),
    }
}
