use crate::{
    types::State,
    utils::{crop, encode_jpg, encode_png, get, get_skin_bytes, resize, set},
};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use base64::{engine::general_purpose, Engine};
use dotenv::dotenv;
use image::imageops;
use reqwest::StatusCode;
use uuid::Uuid;
mod types;
mod utils;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/avatar/{uuid}/{size}/{helm}")]
async fn get_avatar(path: web::Path<(Uuid, u32, bool)>, data: web::Data<State>) -> impl Responder {
    let identifier = path.0.to_string().split_off(30) + &path.2.to_string();
    let mut con = data.connection.clone();
    let size = path.1;

    let key: Result<String, _> = get(&identifier, &mut con).await;
    match key {
        Ok(key) => {
            if size == 8 {
                let jpeg_buffer = general_purpose::STANDARD.decode(key.as_bytes()).unwrap();
                return HttpResponse::build(StatusCode::OK)
                    .content_type("image/jpeg")
                    .body(jpeg_buffer);
            }

            // resize the 8px avatar to the proper size
            let jpeg_buffer = general_purpose::STANDARD.decode(key.as_bytes()).unwrap();
            let avatar = image::load_from_memory(&jpeg_buffer).unwrap().to_rgba8();
            let avatar = resize(avatar, size);
            let jpeg_buffer = encode_jpg(image::DynamicImage::ImageRgba8(avatar), size);
            println!("Cache hit!");

            return HttpResponse::build(StatusCode::OK)
                .content_type("image/jpeg")
                .body(jpeg_buffer);
        }
        Err(_) => {
            println!("Cache miss!");
        }
    }

    let skin = get_skin_bytes(path.0).await;
    let mut avatar = crop(skin.clone(), 8, 8, 8, 8);

    if path.2 == true {
        let helm = crop(skin, 40, 8, 8, 8);
        imageops::overlay(&mut avatar, &helm, 0, 0);
    }

    let mut jpeg_buffer: Vec<u8> = encode_jpg(image::DynamicImage::ImageRgba8(avatar.clone()), 8);

    // If avatar is not 8px wide
    if avatar.width() != size {
        // Cache the 8px avatar
        let b64_jpeg = general_purpose::STANDARD.encode(&jpeg_buffer);
        set(identifier.clone(), b64_jpeg, &mut con).await;

        // Now resize it to the proper size
        avatar = resize(avatar, size);
    } else {
        return HttpResponse::build(StatusCode::OK)
            .content_type("image/jpeg")
            .body(jpeg_buffer);
    }

    jpeg_buffer = encode_jpg(image::DynamicImage::ImageRgba8(avatar), size);

    HttpResponse::build(StatusCode::OK)
        .content_type("image/jpeg")
        .body(jpeg_buffer)
}

#[get("/skin/{uuid}/{size}")]
async fn get_skin(path: web::Path<(Uuid, u32)>) -> impl Responder {
    let size = path.1;
    let skin = get_skin_bytes(path.0).await;
    let mut skin = image::load_from_memory(&skin).unwrap().to_rgba8();

    if size != 64 {
        skin = imageops::resize(&skin, size, size, imageops::FilterType::Nearest);
    }

    let png_buffer = encode_png(image::DynamicImage::ImageRgba8(skin), size);
    HttpResponse::build(StatusCode::OK)
        .content_type("image/png")
        .body(png_buffer)
}

#[get("/skin/{uuid}")]
async fn get_skin_64(path: web::Path<Uuid>) -> impl Responder {
    let skin = get_skin_bytes(path.into_inner()).await;
    let skin = image::load_from_memory(&skin).unwrap().to_rgba8();

    let png_buffer = encode_png(image::DynamicImage::ImageRgba8(skin), 64);
    HttpResponse::build(StatusCode::OK)
        .content_type("image/png")
        .body(png_buffer)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let connection_string = format!(
        "redis://{}:{}@{}",
        std::env::var("REDIS_USERNAME").unwrap(),
        std::env::var("REDIS_PASSWORD").unwrap(),
        std::env::var("REDIS_ADDRESS").unwrap()
    );
    let client = redis::Client::open(connection_string).unwrap();

    let con = client.get_multiplexed_async_connection().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(State {
                connection: con.clone(),
            }))
            .service(hello)
            .service(get_avatar)
            .service(get_skin)
            .service(get_skin_64)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
