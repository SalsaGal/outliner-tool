use std::path::Path;

use egui::{Ui, Context, ColorImage};
use egui_extras::RetainedImage;
use image::{RgbaImage, EncodableLayout};

pub struct Picture {
    source: RgbaImage,
    drawn: RetainedImage,
}

impl Picture {
    pub fn new<P: AsRef<Path>>(path: P) -> Option<Self> {
        let source = image::open(path).ok()?.to_rgba8();
        let drawn = RetainedImage::from_color_image("", ColorImage::from_rgba_unmultiplied(
            [source.dimensions().0 as usize, source.dimensions().1 as usize],
            source.as_flat_samples().as_slice(),
        ));
        Some(Self {
            source,
            drawn,
        })
    }

    pub fn draw(&self, ui: &mut Ui, ctx: &Context) {
        ui.image(self.drawn.texture_id(ctx), self.drawn.size_vec2());
    }
}
