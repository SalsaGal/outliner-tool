use std::path::Path;

use egui::{Ui, Context, ColorImage};
use egui_extras::RetainedImage;
use image::{RgbaImage, Rgba};

pub struct Picture {
    source: RgbaImage,
    drawn: RetainedImage,
}

impl Picture {
    pub fn new<P: AsRef<Path>>(path: P, filter: &Filter) -> Option<Self> {
        let source = image::open(path).ok()?.to_rgba8();
        let filtered = filter.on_source(&source);
        let drawn = RetainedImage::from_color_image("", ColorImage::from_rgba_unmultiplied(
            [filtered.dimensions().0 as usize, filtered.dimensions().1 as usize],
            filtered.as_flat_samples().as_slice(),
        ));
        Some(Self {
            source,
            drawn,
        })
    }

    pub fn update(&mut self, filter: &Filter) {
        let filtered = filter.on_source(&self.source);
        self.drawn = RetainedImage::from_color_image("", ColorImage::from_rgba_unmultiplied(
            [filtered.dimensions().0 as usize, filtered.dimensions().1 as usize],
            filtered.as_flat_samples().as_slice(),
        ));
    }

    pub fn draw(&self, ui: &mut Ui, ctx: &Context) {
        ui.image(self.drawn.texture_id(ctx), self.drawn.size_vec2());
    }
}

pub struct Filter {
    pub sensitivity: u8,
}

impl Filter {
    fn on_source(&self, source: &RgbaImage) -> RgbaImage {
        RgbaImage::from_fn(source.width(), source.height(), |x, y| {
            let original = source.get_pixel(x, y);
            if original.0[3] < self.sensitivity {
                Rgba([0, 0, 0, 0])
            } else {
                Rgba([0, 0, 0, 255])
            }
        })
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            sensitivity: 128,
        }
    }
}
