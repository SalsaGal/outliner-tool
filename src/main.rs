mod picture;

use std::path::Path;

use eframe::App;
use egui::Slider;
use picture::{Picture, Filter};
use rfd::FileDialog;

fn main() {
    eframe::run_native(
        "Outliner",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(ProcessApp::new())),
    )
}

struct ProcessApp {
    pictures: Vec<Picture>,
    filter: Filter,
}

impl ProcessApp {
    pub fn new() -> Self {
        Self {
            pictures: Vec::new(),
            filter: Filter::default(),
        }
    }

    fn load_image<P: AsRef<Path>>(&mut self, path: P) {
        self.pictures.push(Picture::new(path, &self.filter).unwrap());
    }

    fn update_filtered(&mut self) {
        for picture in &mut self.pictures {
            picture.update(&self.filter);
        }
    }
}

impl App for ProcessApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
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
                self.pictures.clear();
            }

            let mut sensitivity = self.filter.sensitivity;
            ui.label("Sensitivity");
            ui.add(Slider::new(&mut sensitivity, 0..=255));

            if sensitivity != self.filter.sensitivity {
                self.filter.sensitivity = sensitivity;
                self.update_filtered();
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                for picture in &self.pictures {
                    picture.draw(ui, ctx);
                }
            });
        });
    }
}
