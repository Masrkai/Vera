use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

const PREVIEW_MAX_WIDTH: u32 = 520; // ~2x the 260px-wide preview box, for HiDPI
const PREVIEW_MAX_HEIGHT: u32 = 360; // ~2x the 180px-tall preview box
const MAXIMIZED_MAX_WIDTH: u32 = 1920;
const MAXIMIZED_MAX_HEIGHT: u32 = 1080;

pub fn load_preview_thumbnail(path: &str) -> Image {
    let decoded = match image::open(path) {
        Ok(img) => img,
        Err(_) => return Image::default(),
    };

    let thumbnail = decoded
        .thumbnail(PREVIEW_MAX_WIDTH, PREVIEW_MAX_HEIGHT)
        .to_rgba8();
    let (width, height) = thumbnail.dimensions();
    let buffer =
        SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(thumbnail.as_raw(), width, height);
    Image::from_rgba8(buffer)
}

pub fn load_maximized_image(path: &str) -> Image {
    let decoded = match image::open(path) {
        Ok(img) => img,
        Err(_) => return Image::default(),
    };

    let thumbnail = decoded
        .thumbnail(MAXIMIZED_MAX_WIDTH, MAXIMIZED_MAX_HEIGHT)
        .to_rgba8();
    let (width, height) = thumbnail.dimensions();
    let buffer =
        SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(thumbnail.as_raw(), width, height);
    Image::from_rgba8(buffer)
}
