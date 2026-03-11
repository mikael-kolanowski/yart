use eframe::egui::{self, Context};
use std::path::PathBuf;

use super::property_editors;
use super::shortcuts::Shortcuts;
use super::utils;
use crate::config::{MaterialConfig, ObjectConfig};
use crate::math::Vec3;

pub struct HelpDialog {
    open: bool,
}

impl HelpDialog {
    pub fn new() -> Self {
        Self { open: false }
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn show(&mut self, ctx: &Context, shortcuts: &Shortcuts) {
        if !self.open {
            return;
        }

        let mut dialog_open = self.open;
        let mut close_requested = false;

        egui::Window::new("Keyboard Shortcuts")
            .open(&mut dialog_open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .spacing([20.0, 5.0])
                    .show(ui, |ui| {
                        // Render Preview shortcut
                        let render_shortcut_str =
                            ctx.format_shortcut(&shortcuts.render_preview.shortcut);
                        ui.label("Render Preview:");
                        ui.label(render_shortcut_str);
                        ui.end_row();

                        // Show Help shortcut
                        let help_shortcut_str = ctx.format_shortcut(&shortcuts.show_help.shortcut);
                        ui.label("Show Help:");
                        ui.label(help_shortcut_str);
                        ui.end_row();
                    });

                ui.separator();
                if ui.button("Close").clicked() {
                    close_requested = true;
                }
            });

        if close_requested {
            self.open = false;
        } else {
            self.open = dialog_open;
        }
    }
}

pub struct AddObjectDialog {
    open: bool,
    pending: Option<ObjectConfig>,
    error: Option<String>,
}

impl AddObjectDialog {
    pub fn new() -> Self {
        Self {
            open: false,
            pending: None,
            error: None,
        }
    }

    pub fn open(&mut self, materials: &[MaterialConfig]) {
        let default_material = materials
            .first()
            .map(|m| m.name().to_string())
            .unwrap_or_default();

        self.pending = Some(ObjectConfig::Sphere {
            position: Vec3::new(0.0, 0.0, -1.0),
            radius: 1.0,
            material: default_material,
        });
        self.error = None;
        self.open = true;
    }

    pub fn show(&mut self, ctx: &Context, materials: &[MaterialConfig]) -> Option<ObjectConfig> {
        if !self.open {
            return None;
        }

        let mut result: Option<ObjectConfig> = None;
        let mut close_requested = false;
        let mut egui_open = self.open;

        egui::Window::new("Add Object")
            .open(&mut egui_open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(ref mut obj) = self.pending {
                    object_type_selector(ui, obj, materials);
                    property_editors::object(ui, obj, materials);
                }

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        close_requested = true;
                    }
                    if ui.button("Add").clicked() {
                        if let Some(ref obj) = self.pending {
                            if let Err(e) = utils::validate_object(obj, materials) {
                                self.error = Some(e);
                            } else {
                                result = Some(obj.clone());
                                close_requested = true;
                            }
                        }
                    }
                });

                if let Some(ref error) = self.error {
                    ui.colored_label(egui::Color32::RED, error);
                }
            });

        if close_requested {
            self.open = false;
        } else {
            self.open = egui_open;
        }

        result
    }
}

pub struct AddMaterialDialog {
    open: bool,
    pending: Option<MaterialConfig>,
    error: Option<String>,
}

impl AddMaterialDialog {
    pub fn new() -> Self {
        Self {
            open: false,
            pending: None,
            error: None,
        }
    }

    pub fn open(&mut self, materials: &[MaterialConfig]) {
        self.pending = Some(MaterialConfig::Lambertian {
            name: utils::new_material_name("lambertian", materials),
            albedo: Vec3::new(0.5, 0.5, 0.5),
        });
        self.error = None;
        self.open = true;
    }

    pub fn show(&mut self, ctx: &Context, materials: &[MaterialConfig]) -> Option<MaterialConfig> {
        if !self.open {
            return None;
        }

        let mut result: Option<MaterialConfig> = None;
        let mut close_requested = false;
        let mut egui_open = self.open;

        egui::Window::new("Add Material")
            .open(&mut egui_open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(ref mut mat) = self.pending {
                    material_type_selector(ui, mat, materials);
                    property_editors::material(ui, mat);
                }

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        close_requested = true;
                    }
                    if ui.button("Add").clicked() {
                        if let Some(ref mat) = self.pending {
                            if let Err(e) = utils::validate_material(mat, materials) {
                                self.error = Some(e);
                            } else {
                                result = Some(mat.clone());
                                close_requested = true;
                            }
                        }
                    }
                });

                if let Some(ref error) = self.error {
                    ui.colored_label(egui::Color32::RED, error);
                }
            });

        if close_requested {
            self.open = false;
        } else {
            self.open = egui_open;
        }

        result
    }
}

fn object_type_selector(ui: &mut egui::Ui, obj: &mut ObjectConfig, materials: &[MaterialConfig]) {
    let current_type = match obj {
        ObjectConfig::Sphere { .. } => "sphere",
        ObjectConfig::Triangle { .. } => "triangle",
        ObjectConfig::Mesh { .. } => "mesh",
    };
    let mut obj_type = current_type.to_string();

    let default_material = materials
        .first()
        .map(|m| m.name().to_string())
        .unwrap_or_default();

    ui.horizontal(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt("object_type_dialog")
            .selected_text(&obj_type)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut obj_type, "sphere".to_string(), "Sphere");
                ui.selectable_value(&mut obj_type, "mesh".to_string(), "Mesh");
            });
    });

    if obj_type != current_type {
        *obj = match obj_type.as_str() {
            "sphere" => ObjectConfig::Sphere {
                position: Vec3::new(0.0, 0.0, -1.0),
                radius: 1.0,
                material: default_material,
            },
            "mesh" => ObjectConfig::Mesh {
                path: PathBuf::new(),
                material: default_material,
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
    let current_type = utils::material_label(&mat);

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
        let default_name = utils::new_material_name(&mat_type, existing);
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
