use eframe::egui;

/// Represents a keyboard shortcut
pub struct Shortcut {
    pub shortcut: egui::KeyboardShortcut,
}

impl Shortcut {
    pub const fn new(shortcut: egui::KeyboardShortcut) -> Self {
        Self { shortcut }
    }
}

/// Collection of all application keyboard shortcuts
pub struct Shortcuts {
    pub render_preview: Shortcut,
    pub show_help: Shortcut,
    pub load_scene: Shortcut,
    pub save_scene: Shortcut,
}

impl Shortcuts {
    pub fn new() -> Self {
        Self {
            render_preview: Shortcut::new(egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::R,
            )),

            show_help: Shortcut::new(egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::F1,
            )),

            load_scene: Shortcut::new(egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::O,
            )),
            save_scene: Shortcut::new(egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::S,
            )),
        }
    }

    /// Check if a shortcut was pressed
    pub fn is_pressed(&self, ctx: &egui::Context, shortcut: &Shortcut) -> bool {
        ctx.input_mut(|i| i.consume_shortcut(&shortcut.shortcut))
    }
}
