use std::path::Path;

use image::Rgba32FImage;

pub struct Picture {
    source: Rgba32FImage,
}

impl Picture {
    pub fn new<P: AsRef<Path>>(path: P) -> Option<Self> {
        let source = image::open(path).ok()?.to_rgba32f();
        Some(Self {
            source,
        })
    }
}
