mod picture;

#[cfg(not(target_family = "wasm"))]
use std::{
    fs::{read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
#[cfg(target_family = "wasm")]
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::Result;
use eframe::App;
use egui::{color_picker::color_edit_button_rgba, Rgba, Slider};
use picture::{Filter, Picture};
#[cfg(not(target_family = "wasm"))]
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use {pollster::block_on, rfd::AsyncFileDialog};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(target_family = "wasm"))]
fn main() {
    eframe::run_native(
        "Outliner",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(ProcessApp::new())),
    );
}

#[cfg(target_family = "wasm")]
fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "canvas",
            eframe::WebOptions::default(),
            Box::new(|_cc| Box::new(ProcessApp::new())),
        )
        .await
        .unwrap();
    });
}

struct ProcessApp {
    pictures: Vec<Picture>,
    filter: Filter,
    #[cfg(not(target_family = "wasm"))]
    config: Config,
    scale: f32,
    errors: Vec<(String, Instant)>,
}

impl ProcessApp {
    pub fn new() -> Self {
        Self {
            pictures: Vec::new(),
            filter: Filter::default(),
            #[cfg(not(target_family = "wasm"))]
            config: Config::new().unwrap_or_default(),
            scale: 1.0,
            errors: Vec::new(),
        }
    }

    fn load_image<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.pictures.push(Picture::new(path, &self.filter)?);
        Ok(())
    }

    fn update_filtered(&mut self) {
        for picture in &mut self.pictures {
            picture.update(&self.filter);
        }
    }

    fn error_hide() -> Instant {
        Instant::now() + Duration::from_secs(5)
    }
}

impl App for ProcessApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        for file in &ctx.input().raw.dropped_files {
            if let Some(path) = &file.path {
                if let Err(err) = self.load_image(path) {
                    self.errors.push((err.to_string(), Self::error_hide()));
                }
            }
        }

        egui::SidePanel::left("actions").show(ctx, |ui| {
            ui.heading("Outliner");
            ui.label(format!("v{VERSION}"));
            if ui.button("Open images").clicked() {
                #[cfg(not(target_family = "wasm"))]
                if let Some(paths) = FileDialog::new().pick_files() {
                    for path in paths {
                        if let Err(err) = self.load_image(path) {
                            self.errors.push((err.to_string(), Self::error_hide()));
                        }
                    }
                }
                #[cfg(target_family = "wasm")]
                if let Some(paths) = block_on(AsyncFileDialog::new().pick_files()) {
                    for path in paths {
                        if let Err(err) = self.load_image(PathBuf::from(
                            String::from_utf8(block_on(path.read())).unwrap(),
                        )) {
                            self.errors.push((err.to_string(), Self::error_hide()));
                        }
                    }
                }
            }
            if ui.button("Clear images").clicked() {
                self.pictures.clear();
            }
            #[cfg(not(target_family = "wasm"))]
            if ui.button("Save images").clicked() {
                if let Some(folder) = FileDialog::new().pick_folder() {
                    for (index, picture) in self.pictures.iter().enumerate() {
                        let mut path = folder.clone();
                        path.push(format!("{index}.png"));
                        if let Err(err) = picture.filtered.save(path) {
                            self.errors.push((err.to_string(), Self::error_hide()));
                        }
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
            #[cfg(not(target_family = "wasm"))]
            if ui.button("Save settings").clicked() {
                if let Some(mut path) = FileDialog::new().add_filter("json", &["json"]).save_file()
                {
                    path.set_extension("json");
                    let settings = serde_json::to_string_pretty(&self.filter).unwrap();
                    match File::create(&path) {
                        Ok(mut file) => {
                            write!(file, "{settings}").unwrap();
                            self.config.last_filter = Some(path);
                            if let Err(err) = self.config.save() {
                                self.errors.push((err.to_string(), Self::error_hide()));
                            }
                        }
                        Err(err) => self.errors.push((err.to_string(), Self::error_hide())),
                    }
                }
            }
            #[cfg(not(target_family = "wasm"))]
            if ui.button("Load settings").clicked() {
                if let Some(path) = FileDialog::new().add_filter("json", &["json"]).pick_file() {
                    match Filter::new(&path) {
                        Ok(filter) => {
                            self.filter = filter;
                            self.config.last_filter = Some(path);
                            if let Err(err) = self.config.save() {
                                self.errors.push((err.to_string(), Self::error_hide()));
                            }
                            filter_changed = true;
                        }
                        Err(err) => self.errors.push((err.to_string(), Self::error_hide())),
                    }
                }
            }
            #[cfg(not(target_family = "wasm"))]
            if let Some(last_filter) = &self.config.last_filter {
                if ui.button("Load last settings").clicked() {
                    match Filter::new(last_filter) {
                        Ok(filter) => {
                            self.filter = filter;
                            filter_changed = true;
                        }
                        Err(err) => self.errors.push((err.to_string(), Self::error_hide())),
                    }
                }
            }
            if filter_changed {
                self.update_filtered();
            }

            for (error, _) in &self.errors {
                ui.label(error);
            }
            self.errors = self
                .errors
                .clone()
                .into_iter()
                .filter(|(_, time)| *time >= Instant::now())
                .collect();
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
    error_time_secs: u16,
}

#[cfg(not(target_family = "wasm"))]
impl Config {
    pub fn new() -> Result<Self> {
        if let Ok(contents) = read_to_string(Self::path()?) {
            Ok(serde_json::from_str(&contents).unwrap_or_default())
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let mut file = File::create(Self::path()?)?;
        let contents = serde_json::to_string_pretty(&self)?;
        write!(file, "{contents}")?;
        Ok(())
    }

    fn path() -> Result<PathBuf> {
        let mut path = dirs_next::config_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
        path.push("outliner");
        std::fs::create_dir_all(&path)?;
        path.push("config.json");
        Ok(path)
    }
}
