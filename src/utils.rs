use crate::types::{DecodedProperty, IsEmpty, MojangProfile};
use base64::{engine::general_purpose, Engine};
use image::{
    codecs::png::PngEncoder, imageops, EncodableLayout, ImageBuffer, ImageEncoder, Pixel, Rgb, Rgba,
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

pub fn resize<T: Pixel + 'static>(
    image: &ImageBuffer<T, Vec<T::Subpixel>>,
    size: u32,
) -> ImageBuffer<T, Vec<T::Subpixel>> {
    let returnable = imageops::resize(image, size, size, imageops::FilterType::Nearest);
    returnable
}

pub async fn set<T: redis::ToRedisArgs + Send + Sync>(
    k: String,
    v: T,
    con: &mut redis::aio::MultiplexedConnection,
) {
    let _: () = con.set(k, v).await.unwrap();
}

pub async fn get<T: redis::FromRedisValue + IsEmpty>(
    k: &String,
    con: &mut redis::aio::MultiplexedConnection,
) -> redis::RedisResult<T> {
    let v: redis::RedisResult<T> = con.get(k).await;
    match &v {
        Ok(value) => {
            if value.is_empty() {
                return Err(redis::RedisError::from((
                    redis::ErrorKind::TypeError,
                    "Value is empty",
                )));
            }
        }
        Err(_) => {}
    }
    v
}

pub fn get_id(uuid: Uuid, helm: bool) -> String {
    let identifier = format!(
        "{}-{}",
        uuid.to_string().split_off(30),
        helm.to_string()[0..1].to_string()
    );
    identifier
}

pub fn encode_png(
    image: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
    size: u32,
    color_type: image::ColorType,
) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut new_image: ImageBuffer<Rgb<u8>, Vec<u8>>;
    let encoder = PngEncoder::new_with_quality(
        &mut buffer,
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::NoFilter,
    );

    // Handle converting to RGB if the image is RGBA
    if color_type == image::ColorType::Rgb8 {
        if image.as_bytes().len() as i32 != 192 {
            new_image = ImageBuffer::new(size, size);
            for (x, y, pixel) in image.enumerate_pixels() {
                new_image.put_pixel(x, y, Rgb([pixel[0], pixel[1], pixel[2]]));
            }
            encoder
                .write_image(&new_image, size, size, color_type)
                .unwrap();
            return buffer;
        }
    }
    encoder.write_image(&image, size, size, color_type).unwrap();
    buffer
}

pub async fn get_skin_bytes(uuid: Uuid) -> Result<Vec<u8>, &'static str> {
    let mojang_url = "https://sessionserver.mojang.com/session/minecraft/profile/";
    let resp = reqwest::get(mojang_url.to_string() + &uuid.to_string())
        .await
        .unwrap();
    let resp_result: Result<MojangProfile, reqwest::Error> = resp.json().await;
    let profile = match resp_result {
        Ok(profile) => profile,
        Err(_) => {
            // TODO add long term storage for skins
            return Err("Error getting profile from Mojang");
        }
    };
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
    Ok(skin.to_vec())
}
