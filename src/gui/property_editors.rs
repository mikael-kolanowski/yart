use std::path::PathBuf;

use eframe::egui::{self};

use crate::{MaterialConfig, ObjectConfig, gui::editor::ViewportRendererConfig, math::Vec3};

use super::widgets;

pub fn object(ui: &mut egui::Ui, obj: &mut ObjectConfig, materials: &[MaterialConfig]) {
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
                    widgets::vector_input(ui, position);
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

pub fn material(ui: &mut egui::Ui, mat: &mut MaterialConfig) {
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
                widgets::color_input(ui, albedo);
            }
            MaterialConfig::Metal { name, albedo, fuzz } => {
                ui.label("Type:");
                ui.label("Metal");
                ui.end_row();

                ui.label("Name:");
                ui.text_edit_singleline(name);
                ui.end_row();

                ui.label("Albedo:");
                widgets::color_input(ui, albedo);
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

pub fn camera(ui: &mut egui::Ui, camera: &mut crate::config::CameraConfig) {
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
                widgets::vector_input(ui, &mut camera.position.0);
            });
            ui.end_row();

            ui.label("Look At:");
            ui.horizontal(|ui| {
                widgets::vector_input(ui, &mut camera.look_at.0);
            });
            ui.end_row();
        });
}

pub fn renderer(ui: &mut egui::Ui, renderer: &mut crate::config::RendererConfig) {
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

pub fn image(ui: &mut egui::Ui, image: &mut crate::config::ImageConfig) {
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

pub fn sky(ui: &mut egui::Ui, sky: &mut crate::config::SkyConfig) {
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
                    widgets::color_input(ui, from);
                    ui.end_row();

                    ui.label("To:");
                    widgets::color_input(ui, to);
                    ui.end_row();
                }
                crate::config::SkyConfig::Solid { color } => {
                    ui.label("Color:");
                    widgets::color_input(ui, color);
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

pub fn viewport(ui: &mut egui::Ui, viewport: &mut ViewportRendererConfig) {
    egui::Grid::new("viewport_config_grid")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Preview Samples:");
            ui.add(egui::DragValue::new(&mut viewport.samples_per_pixel).speed(1.0));
            ui.end_row();

            ui.label("Preview Bounces:");
            ui.add(egui::DragValue::new(&mut viewport.max_bounces).speed(1.0));
            ui.end_row();
        });
}
