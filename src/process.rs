use image::DynamicImage;

pub struct Process {
    image: DynamicImage,
}

impl Process {
    pub fn new(image: DynamicImage) -> Option<Self> {
        Some(Self {
            image,
        })
    }

    pub fn process(&self) -> DynamicImage {
        self.image.clone()
    }
}
