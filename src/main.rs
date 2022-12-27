mod picture;

use std::{
    fs::{read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
};

use eframe::App;
use egui::{color_picker::color_edit_button_rgba, Rgba, Slider};
use picture::{Filter, Picture};
use rfd::FileDialog;
use serde::{Serialize, Deserialize};

const VERSION: &str = "0.2.1";

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
    config: Config,
    scale: f32,
}

impl ProcessApp {
    pub fn new() -> Self {
        Self {
            pictures: Vec::new(),
            filter: Filter::default(),
            config: Config::new(),
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

            let mut filter_changed = false;
            if sensitivity != self.filter.sensitivity {
                self.filter.sensitivity = sensitivity;
                filter_changed = true;
            }
            if outline != self.filter.outline {
                self.filter.outline = outline;
                filter_changed = true;
            }
            if background != self.filter.background {
                self.filter.background = background;
                filter_changed = true;
            }
            if ui.button("Save settings").clicked() {
                if let Some(mut path) = FileDialog::new().add_filter("json", &["json"]).save_file()
                {
                    path.set_extension("json");
                    let settings = serde_json::to_string_pretty(&self.filter).unwrap();
                    let mut file = File::create(&path).unwrap();
                    write!(file, "{settings}").unwrap();
                    self.config.last_filter = Some(path);
                    self.config.save();
                }
            }
            if ui.button("Load settings").clicked() {
                if let Some(path) = FileDialog::new().add_filter("json", &["json"]).pick_file() {
                    self.filter = Filter::new(&path);
                    self.config.last_filter = Some(path);
                    self.config.save();
                    filter_changed = true;
                }
            }
            if let Some(last_filter) = &self.config.last_filter {
                if ui.button("Load last settings").clicked() {
                    self.filter = Filter::new(last_filter);
                    filter_changed = true;
                }
            }
            if filter_changed {
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

#[derive(Default, Serialize, Deserialize)]
struct Config {
    last_filter: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        if let Ok(contents) = read_to_string(Self::path()) {
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let mut file = File::create(Self::path()).unwrap();
        let contents = serde_json::to_string_pretty(&self).unwrap();
        write!(file, "{contents}").unwrap();
    }

    fn path() -> PathBuf {
        let mut path = dirs_next::data_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
        path.push("outliner");
        std::fs::create_dir_all(&path).unwrap();
        path.push("config.json");
        path
    }
}
