use crate::types::UuidOrString;
use image::{imageops, DynamicImage::ImageRgb8, DynamicImage::ImageRgba8};
use reqwest::StatusCode;
use uuid::Uuid;
use worker::ResponseBody::Body;
use worker::*;
mod bytes;
mod cache;
mod img;
mod mojang;
mod types;

async fn health(_: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("OK")
}

async fn get_avatar(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let uuid: UuidOrString = ctx.param("uuid").unwrap().parse().unwrap();
    let size: u32 = ctx.param("size").unwrap().parse().unwrap();
    let helm: bool = ctx.param("helm").unwrap().parse().unwrap();

    let mut headers = Headers::new();
    let _ = headers.append("Cache-Control", "max-age=1200");
    let _ = headers.append("Content-Type", "image/png");

    let identifier = cache::create_id(uuid.clone(), helm);

    if size > 512 || size < 8 || size % 8 != 0 {
        return Response::error("Size must be between 8 and 512, and divisible by 8.", 400);
    }

    // STEP: Check if the avatar is cached, if so, load it and return it
    let avatars = match ctx.kv("AVATARS") {
        Ok(avatars) => avatars,
        Err(_) => {
            return Response::error("Error getting KV namespace.", 500);
        }
    };

    match avatars.get(&identifier).bytes().await.unwrap() {
        Some(mut buffer) => {
            buffer = bytes::repair(buffer);
            let avatar = match image::load_from_memory(&buffer) {
                Ok(avatar) => avatar.to_rgb8(),
                Err(_) => {
                    return Response::error("Error loading avatar from cache!", 500);
                }
            };

            if size != 8 {
                let avatar = img::resize(&avatar, size);
                buffer = img::encode_png(ImageRgb8(avatar));
            }

            return Ok(Response::from_body(Body(buffer))
                .unwrap()
                .with_headers(headers));
        }
        None => {}
    };

    let uuid = match uuid {
        UuidOrString::Uuid(uuid) => uuid,
        UuidOrString::String(username) => match mojang::get_uuid(username).await {
            uuid if uuid != Uuid::nil() => uuid,
            _ => {
                return Response::error("User not found!", 404);
            }
        },
    };

    let skin = match mojang::get_skin(uuid).await {
        Ok(skin) => skin,
        Err(_) => {
            return Response::error("Skin not found!", 404);
        }
    };

    let mut avatar = img::crop(skin.clone(), 8, 8, 8, 8);
    if helm == true {
        let helm = img::crop(skin.to_vec(), 40, 8, 8, 8);
        imageops::overlay(&mut avatar, &helm, 0, 0);
    }

    let mut avatar = ImageRgba8(avatar).to_rgb8();
    let buffer: Vec<u8> = img::encode_png(ImageRgb8(avatar.clone()));
    if avatar.width() != size {
        avatar = img::resize(&avatar, size);
    };

    avatars
        .put_bytes(&identifier, &bytes::strip(buffer))
        .unwrap()
        .execute()
        .await
        .unwrap();

    Ok(
        Response::from_body(Body(img::encode_png(ImageRgb8(avatar))))
            .unwrap()
            .with_headers(headers),
    )
}

async fn get_skin(_: Request, ctx: RouteContext<()>) -> Result<Response> {
    let uuid: UuidOrString = ctx.param("uuid").unwrap().parse().unwrap();
    let mut headers = Headers::new();
    let _ = headers.append("Cache-Control", "max-age=1200");
    let _ = headers.append("Content-Type", "image/png");

    let uuid = match uuid {
        UuidOrString::Uuid(uuid) => uuid,
        UuidOrString::String(username) => match mojang::get_uuid(username).await {
            uuid if uuid != Uuid::nil() => uuid,
            _ => {
                return Response::error("User not found!", 404);
            }
        },
    };

    let skin = match mojang::get_skin(uuid).await {
        Ok(skin) => skin,
        Err(_) => {
            return Response::error("Skin not found!", 404);
        }
    };

    let skin = image::load_from_memory(&skin).unwrap().to_rgba8();

    let buffer = img::encode_png(ImageRgba8(skin));
    Ok(Response::from_body(Body(buffer))
        .unwrap()
        .with_headers(headers))
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    Router::new()
        .get_async("/", health)
        .get_async("/avatar/:uuid/:size/:helm", get_avatar)
        .get_async("/skin/:uuid", get_skin)
        .run(req, env)
        .await
}
