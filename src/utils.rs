use crate::types::{DecodedProperty, MojangProfile};
use base64::{engine::general_purpose, Engine};
use image::{
    codecs::{jpeg::JpegEncoder, png::PngEncoder},
    imageops, ImageBuffer, ImageEncoder, Rgba,
};
use redis::AsyncCommands;
use uuid::Uuid;

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

pub fn resize(image: ImageBuffer<Rgba<u8>, Vec<u8>>, size: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let returnable = imageops::resize(&image, size, size, imageops::FilterType::Nearest);
    returnable
}

pub async fn set(k: String, v: String, con: &mut redis::aio::MultiplexedConnection) {
    let _: () = con.set(k, v).await.unwrap();
}

pub async fn get(
    k: &String,
    con: &mut redis::aio::MultiplexedConnection,
) -> redis::RedisResult<String> {
    let v: redis::RedisResult<String> = con.get(k).await;
    v
}

pub fn encode_jpg(image: image::DynamicImage, size: u32) -> Vec<u8> {
    let mut jpeg_buffer = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_buffer, 100);
    if image.width() != size {
        println!("width != size {} != {}", image.width(), size);
    }
    encoder
        .encode(&image.into_rgba8(), size, size, image::ColorType::Rgba8)
        .unwrap();
    jpeg_buffer
}

pub fn encode_png(image: image::DynamicImage, size: u32) -> Vec<u8> {
    let mut jpeg_buffer = Vec::new();
    let encoder = PngEncoder::new_with_quality(
        &mut jpeg_buffer,
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::NoFilter,
    );
    encoder
        .write_image(&image.into_rgba8(), size, size, image::ColorType::Rgba8)
        .unwrap();
    jpeg_buffer
}

pub async fn get_skin_bytes(uuid: Uuid) -> Vec<u8> {
    let mojang_url = "https://sessionserver.mojang.com/session/minecraft/profile/";
    let resp = reqwest::get(mojang_url.to_string() + &uuid.to_string())
        .await
        .unwrap();
    let profile: MojangProfile = resp.json().await.unwrap();

    let decoded_obj = general_purpose::STANDARD
        .decode(profile.properties[0].value.as_bytes())
        .unwrap();
    let decoded_obj: DecodedProperty = serde_json::from_slice(&decoded_obj).unwrap();
    let skin = reqwest::get(decoded_obj.textures.skin.url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    skin.to_vec()
}
