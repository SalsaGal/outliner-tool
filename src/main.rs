use std::{path::Path, fs::File, io::Read};

use eframe::App;
use egui_extras::RetainedImage;
use image::{DynamicImage, RgbaImage};
use rfd::FileDialog;

fn main() {
    eframe::run_native(
        "Outliner",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(ProcessApp::new())),
    )
}

struct ProcessApp {
    sources: Vec<RgbaImage>,
}

impl ProcessApp {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    fn load_image<P: AsRef<Path>>(&mut self, path: P) {
        let mut image_bytes = Vec::new();
        let mut file = File::open(&path).unwrap();
        file.read_to_end(&mut image_bytes).unwrap();
        let image = image::open(&path).unwrap().to_rgba8();
        self.sources.push(image);
    }
}

impl App for ProcessApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        for file in &ctx.input().raw.dropped_files {
            if let Some(path) = &file.path {
                self.load_image(path);
            }
        }

        egui::SidePanel::left("actions").show(ctx, |ui| {
            ui.heading("Outliner");
            if ui.button("Open images").clicked() {
                if let Some(paths) = FileDialog::new().pick_files() {
                    for path in paths {
                        self.load_image(path);
                    }
                }
            }
            if ui.button("Clear images").clicked() {
                self.sources.clear();
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
            });
        });
    }
}
