use image::{codecs::png::PngEncoder, imageops, ImageBuffer, ImageEncoder, Pixel, Rgba};

pub fn crop(
    image: Vec<u8>,
    offset_x: u32,
    offset_y: u32,
    height: u32,
    width: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = image::load_from_memory(&image).unwrap().to_rgba8();
    let image = imageops::crop(&mut image, offset_x, offset_y, width, height);
    let image = image.to_image();
    image
}

pub fn resize<T: Pixel + 'static>(
    image: &ImageBuffer<T, Vec<T::Subpixel>>,
    size: u32,
) -> ImageBuffer<T, Vec<T::Subpixel>> {
    let returnable = imageops::resize(image, size, size, imageops::FilterType::Nearest);
    returnable
}

pub fn encode_png(mut image: image::DynamicImage) -> Vec<u8> {
    let mut buffer = Vec::new();
    let encoder = PngEncoder::new_with_quality(
        &mut buffer,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::NoFilter,
    );

    if image.color() == image::ColorType::Rgba8 {
        image = image.into_rgb8().into();
    }

    encoder
        .write_image(
            image.as_bytes(),
            image.width(),
            image.height(),
            image.color().into(),
        )
        .unwrap();
    buffer
}
