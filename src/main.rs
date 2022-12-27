mod picture;

use std::{
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};

use eframe::App;
use egui::{color_picker::color_edit_button_rgba, Rgba, Slider};
use picture::{Filter, Picture};
use rfd::FileDialog;

const VERSION: &str = "0.1.0";

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
    scale: f32,
}

impl ProcessApp {
    pub fn new() -> Self {
        Self {
            pictures: Vec::new(),
            filter: Filter::default(),
            scale: 1.0,
        }
    }

    fn load_image<P: AsRef<Path>>(&mut self, path: P) {
        self.pictures
            .push(Picture::new(path, &self.filter).unwrap());
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
            ui.label(format!("v{VERSION}"));
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
            if ui.button("Save images").clicked() {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    for (index, picture) in self.pictures.iter().enumerate() {
                        let mut path = folder.clone();
                        path.push(format!("{index}.png"));
                        picture.filtered.save(path).unwrap();
                    }
                }
            }

            let mut sensitivity = self.filter.sensitivity;
            ui.label("Sensitivity");
            ui.add(Slider::new(&mut sensitivity, 0..=255));

            let mut outline = Rgba::from_srgba_unmultiplied(
                self.filter.outline[0],
                self.filter.outline[1],
                self.filter.outline[2],
                self.filter.outline[3],
            );
            ui.label("Outline");
            color_edit_button_rgba(ui, &mut outline, egui::color_picker::Alpha::Opaque);
            let outline = outline.to_srgba_unmultiplied();

            let mut background = Rgba::from_srgba_unmultiplied(
                self.filter.background[0],
                self.filter.background[1],
                self.filter.background[2],
                self.filter.background[3],
            );
            ui.label("Background");
            color_edit_button_rgba(ui, &mut background, egui::color_picker::Alpha::OnlyBlend);
            let background = background.to_srgba_unmultiplied();

            ui.label("Scale");
            ui.add(Slider::new(&mut self.scale, 0.125..=4.0));

            let mut changed = false;
            if sensitivity != self.filter.sensitivity {
                self.filter.sensitivity = sensitivity;
                changed = true;
            }
            if outline != self.filter.outline {
                self.filter.outline = outline;
                changed = true;
            }
            if background != self.filter.background {
                self.filter.background = background;
                changed = true;
            }
            if ui.button("Save settings").clicked() {
                if let Some(mut path) = FileDialog::new().add_filter("json", &["json"]).save_file()
                {
                    path.set_extension("json");
                    let settings = serde_json::to_string_pretty(&self.filter).unwrap();
                    let mut file = File::create(path).unwrap();
                    write!(file, "{settings}").unwrap();
                }
            } else if ui.button("Load settings").clicked() {
                if let Some(path) = FileDialog::new().add_filter("json", &["json"]).pick_file() {
                    let settings = read_to_string(path).unwrap();
                    self.filter = serde_json::from_str(&settings).unwrap();
                    changed = true;
                }
            }
            if changed {
                self.update_filtered();
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                for picture in &self.pictures {
                    picture.draw(ui, ctx, self.scale);
                }
            });
        });
    }
}
