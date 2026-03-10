use eframe::egui::{self};

use crate::math::Vec3;

pub fn panel_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).strong().heading().size(14.0));
}

pub fn vector_input(ui: &mut egui::Ui, vector: &mut Vec3) {
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut vector.x).speed(0.1))
            .on_hover_text("x");
        ui.add(egui::DragValue::new(&mut vector.y).speed(0.1))
            .on_hover_text("y");
        ui.add(egui::DragValue::new(&mut vector.z).speed(0.1))
            .on_hover_text("z");
    });
}

pub fn color_input(ui: &mut egui::Ui, color: &mut Vec3) {
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
