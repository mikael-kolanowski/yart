use std::path::PathBuf;

use eframe::egui::{self};

use crate::config::{self, ViewportConfig};
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
    pending_object: PendingObject,
    pending_material: PendingMaterial,
}

#[derive(Clone)]
struct PendingObject {
    obj_type: String,
    position: Vec3,
    radius: f64,
    p1: Point3,
    p2: Point3,
    p3: Point3,
    path: String,
    material: String,
    error: Option<String>,
}

impl Default for PendingObject {
    fn default() -> Self {
        Self {
            obj_type: "sphere".to_string(),
            position: Vec3::new(0.0, 0.0, -1.0),
            radius: 0.5,
            p1: Point3::new(0.0, 0.0, 0.0),
            p2: Point3::new(1.0, 0.0, 0.0),
            p3: Point3::new(0.0, 1.0, 0.0),
            path: String::new(),
            material: String::new(),
            error: None,
        }
    }
}

#[derive(Clone)]
struct PendingMaterial {
    mat_type: String,
    name: String,
    albedo: [f32; 3],
    fuzz: f64,
    error: Option<String>,
}

impl Default for PendingMaterial {
    fn default() -> Self {
        Self {
            mat_type: "lambertian".to_string(),
            name: String::new(),
            albedo: [0.0; 3],
            fuzz: 0.3,
            error: None,
        }
    }
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
            pending_object: PendingObject::default(),
            pending_material: PendingMaterial::default(),
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

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    if ui.button("Save").clicked() {
                        println!("TODO: export config");
                    }
                });
            });
        });

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            panel_heading(ui, "Scene");

            ui.horizontal(|ui| {
                if ui.button("+ Add object").clicked() {
                    self.pending_object = PendingObject {
                        material: self
                            .config
                            .materials
                            .first()
                            .map(|m| m.name().to_string())
                            .unwrap_or_default(),
                        ..Default::default()
                    };
                    self.add_object_dialog_open = true;
                }

                if ui.button("+ Add material").clicked() {
                    self.pending_material = PendingMaterial::default();
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
            panel_heading(ui, "Properties");

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
        let mut object_dialog_open = self.add_object_dialog_open;
        if object_dialog_open {
            println!("add object!!!!");
            egui::Window::new("Add Object")
                .open(&mut object_dialog_open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    add_object_dialog(ui, &mut self.pending_object, &self.config.materials);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.add_object_dialog_open = false;
                        }
                        if ui.button("Add").clicked() {
                            if let Some(obj) = validate_and_create_object(
                                &self.pending_object,
                                &self.config.materials,
                            ) {
                                self.config.objects.push(obj);
                                self.add_object_dialog_open = false;
                            } else {
                                self.pending_object.error =
                                    Some("Please fill in all required fields".to_string());
                            }
                        }
                    });

                    if let Some(ref error) = self.pending_object.error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                });
            self.add_object_dialog_open = object_dialog_open;
        }

        // Material Dialog
        let mut material_dialog_open = self.add_material_dialog_open;
        if material_dialog_open {
            egui::Window::new("Add Material")
                .open(&mut material_dialog_open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    add_material_dialog(ui, &mut self.pending_material);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.add_material_dialog_open = false;
                        }
                        if ui.button("Add").clicked() {
                            if let Some(mat) = validate_and_create_material(
                                &self.pending_material,
                                &self.config.materials,
                            ) {
                                self.config.materials.push(mat);
                                self.add_material_dialog_open = false;
                            } else {
                                self.pending_material.error =
                                    Some("Please enter a unique name".to_string());
                            }
                        }
                    });

                    if let Some(ref error) = self.pending_material.error {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                });
            self.add_material_dialog_open = material_dialog_open;
        }
    }
}

fn add_object_dialog(ui: &mut egui::Ui, pending: &mut PendingObject, materials: &[MaterialConfig]) {
    ui.group(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt("object_type")
            .selected_text(&pending.obj_type)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut pending.obj_type, "sphere".to_string(), "Sphere");
                ui.selectable_value(&mut pending.obj_type, "triangle".to_string(), "Triangle");
                ui.selectable_value(&mut pending.obj_type, "mesh".to_string(), "Mesh");
            });

        match pending.obj_type.as_str() {
            "sphere" => {
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    vector_input(ui, &mut pending.position);
                });
                ui.add(
                    egui::DragValue::new(&mut pending.radius)
                        .speed(0.1)
                        .range(0.0..=f64::MAX),
                );
            }
            "mesh" => {
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.text_edit_singleline(&mut pending.path);
                    if ui.button("Browse").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("OBJ files", &["obj"])
                            .pick_file()
                        {
                            pending.path = path.display().to_string();
                        }
                    }
                });
            }
            _ => {}
        }

        ui.label("Material:");
        egui::ComboBox::from_id_salt("object_material")
            .selected_text(pending.material.clone())
            .show_ui(ui, |ui| {
                for mat in materials {
                    ui.selectable_value(&mut pending.material, mat.name().to_string(), mat.name());
                }
            });
    });
}

fn add_material_dialog(ui: &mut egui::Ui, pending: &mut PendingMaterial) {
    ui.group(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt("material_type")
            .selected_text(&pending.mat_type)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut pending.mat_type,
                    "lambertian".to_string(),
                    "Lambertian",
                );
                ui.selectable_value(&mut pending.mat_type, "metal".to_string(), "Metal");
                ui.selectable_value(
                    &mut pending.mat_type,
                    "normal_vis".to_string(),
                    "Normal Visualization",
                );
            });

        ui.text_edit_singleline(&mut pending.name);

        match pending.mat_type.as_str() {
            "lambertian" | "metal" => {
                ui.horizontal(|ui| {
                    ui.label("Albedo:");
                    egui::color_picker::color_edit_button_rgb(ui, &mut pending.albedo);
                    println!("{:?}", pending.albedo);
                });

                if pending.mat_type == "metal" {
                    ui.add(
                        egui::DragValue::new(&mut pending.fuzz)
                            .speed(0.05)
                            .range(0.0..=1.0),
                    );
                }
            }
            _ => {}
        }
    });
}

fn validate_and_create_object(
    pending: &PendingObject,
    materials: &[MaterialConfig],
) -> Option<ObjectConfig> {
    let material_exists = materials.iter().any(|m| m.name() == pending.material);
    if !material_exists {
        return None;
    }

    match pending.obj_type.as_str() {
        "sphere" => Some(ObjectConfig::Sphere {
            position: pending.position,
            radius: pending.radius,
            material: pending.material.clone(),
        }),
        "triangle" => Some(ObjectConfig::Triangle {
            p1: pending.p1,
            p2: pending.p2,
            p3: pending.p3,
            material: pending.material.clone(),
        }),
        "mesh" => {
            if pending.path.is_empty() {
                return None;
            }
            Some(ObjectConfig::Mesh {
                path: PathBuf::from(&pending.path),
                material: pending.material.clone(),
            })
        }
        _ => None,
    }
}

fn validate_and_create_material(
    pending: &PendingMaterial,
    existing: &[MaterialConfig],
) -> Option<MaterialConfig> {
    if pending.name.is_empty() {
        return None;
    }

    let name_unique = !existing.iter().any(|m| m.name() == pending.name);
    if !name_unique {
        return None;
    }

    match pending.mat_type.as_str() {
        "lambertian" => Some(MaterialConfig::Lambertian {
            name: pending.name.clone(),
            albedo: Vec3::new(
                pending.albedo[0] as f64,
                pending.albedo[1] as f64,
                pending.albedo[2] as f64,
            ),
        }),
        "metal" => Some(MaterialConfig::Metal {
            name: pending.name.clone(),
            albedo: Vec3::new(
                pending.albedo[0] as f64,
                pending.albedo[1] as f64,
                pending.albedo[2] as f64,
            ),
            fuzz: pending.fuzz,
        }),
        "normal_vis" => Some(MaterialConfig::NormalVisualization {
            name: pending.name.clone(),
        }),
        _ => None,
    }
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

                ui.label("Radius");
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
                let mut display_path = path.as_mut_os_string().clone().into_string().unwrap();
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
                        // TODO: make the path editable
                        display_path = selected_path.display().to_string();
                    }
                }
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

                ui.label("Name");
                ui.text_edit_singleline(name);
                ui.end_row();

                ui.label("Albedo:");
                color_input(ui, albedo);
                ui.end_row();

                ui.label("Fuzz");
                ui.add(egui::DragValue::new(fuzz).speed(0.05).range(0.0..=1.0));
            }
            _ => {}
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
