use std::vec;

use image::{codecs::png::PngEncoder, imageops, ImageBuffer, ImageEncoder, Pixel, Rgb, Rgba};

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

pub fn convert_to_rgb(image: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    image::RgbImage::from_raw(image.width(), image.height(), image.into_raw()).unwrap()
}

pub fn resize<T: Pixel + 'static>(
    image: &ImageBuffer<T, Vec<T::Subpixel>>,
    size: u32,
) -> ImageBuffer<T, Vec<T::Subpixel>> {
    let returnable = imageops::resize(image, size, size, imageops::FilterType::Nearest);
    returnable
}

pub fn encode_png(image: image::DynamicImage) -> Vec<u8> {
    let mut buffer = Vec::new();
    let encoder = PngEncoder::new_with_quality(
        &mut buffer,
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::NoFilter,
    );

    // Handle converting to RGB if the image is RGBA or RGB
    let color_type = image.color();
    if color_type == image::ColorType::Rgb8 {
        let new_image = image.to_rgb8();
        encoder
            .write_image(
                &new_image,
                new_image.width(),
                new_image.height(),
                image::ColorType::Rgb8,
            )
            .unwrap();
        return buffer;
    } else if color_type == image::ColorType::Rgba8 {
        let new_image = image.to_rgba8();
        encoder
            .write_image(
                &new_image,
                new_image.width(),
                new_image.height(),
                image::ColorType::Rgba8,
            )
            .unwrap();
        return buffer;
    }
    vec![]
}
