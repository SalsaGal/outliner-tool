use std::{path::Path, fs::read_to_string};

use egui::{ColorImage, Context, Ui};
use egui_extras::RetainedImage;
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

pub struct Picture {
    source: RgbaImage,
    pub filtered: RgbaImage,
    drawn: RetainedImage,
}

impl Picture {
    pub fn new<P: AsRef<Path>>(path: P, filter: &Filter) -> Option<Self> {
        let source = image::open(path).ok()?.to_rgba8();
        let filtered = filter.on_source(&source);
        let drawn = RetainedImage::from_color_image(
            "",
            ColorImage::from_rgba_unmultiplied(
                [
                    filtered.dimensions().0 as usize,
                    filtered.dimensions().1 as usize,
                ],
                filtered.as_flat_samples().as_slice(),
            ),
        );
        Some(Self {
            source,
            filtered,
            drawn,
        })
    }

    pub fn update(&mut self, filter: &Filter) {
        self.filtered = filter.on_source(&self.source);
        self.drawn = RetainedImage::from_color_image(
            "",
            ColorImage::from_rgba_unmultiplied(
                [
                    self.filtered.dimensions().0 as usize,
                    self.filtered.dimensions().1 as usize,
                ],
                self.filtered.as_flat_samples().as_slice(),
            ),
        );
    }

    pub fn draw(&self, ui: &mut Ui, ctx: &Context, scale: f32) {
        ui.image(self.drawn.texture_id(ctx), self.drawn.size_vec2() * scale);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Filter {
    pub sensitivity: u8,
    pub outline: [u8; 4],
    pub background: [u8; 4],
}

impl Filter {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let contents = read_to_string(path).unwrap();
        serde_json::from_str(&contents).unwrap()
    }

    fn on_source(&self, source: &RgbaImage) -> RgbaImage {
        RgbaImage::from_fn(source.width(), source.height(), |x, y| {
            let original = source.get_pixel(x, y);
            if original.0[3] < self.sensitivity {
                Rgba(self.background)
            } else {
                Rgba(self.outline)
            }
        })
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            sensitivity: 128,
            outline: [0, 0, 0, 255],
            background: [0, 0, 0, 0],
        }
    }
}
