use std::path::Path;

use eframe::App;
use image::DynamicImage;
use rfd::FileDialog;

fn main() {
    eframe::run_native(
        "Outliner",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(ProcessApp::new())),
    )
}

struct ProcessApp {
    images: Vec<DynamicImage>
}

impl ProcessApp {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
        }
    }

    fn load_image<P: AsRef<Path>>(&mut self, path: P) {}
}

impl App for ProcessApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        for file in &ctx.input().raw.dropped_files {
            if let Some(path) = &file.path {
                self.load_image(path);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Outliner");
            if ui.button("Open images").clicked() {
                if let Some(paths) = FileDialog::new().pick_files() {
                    for path in paths {
                        self.load_image(path);
                    }
                }
            }
            if ui.button("Clear images").clicked() {
                self.images.clear();
            }
        });
    }
}
