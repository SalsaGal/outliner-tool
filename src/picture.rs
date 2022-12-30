use std::{collections::HashMap, fs::read_to_string, path::Path};

use anyhow::Result;
use egui::{ColorImage, Context, Ui};
use egui_extras::RetainedImage;
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

pub struct Picture {
    source: RgbaImage,
    background: [u8; 4],
    pub filtered: RgbaImage,
    drawn: RetainedImage,
}

impl Picture {
    pub fn new<P: AsRef<Path>>(path: P, filter: &Filter) -> Result<Self> {
        let source = image::open(path)?.to_rgba8();
        let color_count = source
            .pixels()
            .fold(HashMap::<_, usize>::new(), |mut acc, x| {
                *acc.entry(x).or_default() += 1;
                acc
            });
        let background = color_count
            .into_iter()
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .unwrap()
            .0
             .0;
        let filtered = filter.on_source(&source, background);
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

        Ok(Self {
            source,
            background,
            filtered,
            drawn,
        })
    }

    pub fn update(&mut self, filter: &Filter) {
        self.filtered = filter.on_source(&self.source, self.background);
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
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    fn on_source(&self, source: &RgbaImage, background: [u8; 4]) -> RgbaImage {
        RgbaImage::from_fn(source.width(), source.height(), |x, y| {
            let original = source.get_pixel(x, y);
            let distance = color_distance(background, original.0) / 4;
            if (distance as u8) < self.sensitivity {
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

fn color_distance(a: [u8; 4], b: [u8; 4]) -> u16 {
    if a[3] | b[3] == 0 {
        return 0;
    }
    let a = a.map(|x| x as u16);
    let b = b.map(|x| x as u16);
    let diffs = a.iter().zip(b).map(|(a, b)| a.abs_diff(b));
    diffs.sum()
}
