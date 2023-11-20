use crate::{
    types::State,
    utils::{crop, encode_png, get, get_skin_bytes, resize, set},
};
use actix_web::{get, http::header, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use image::imageops;
use reqwest::StatusCode;
use uuid::Uuid;
mod types;
mod utils;

#[get("/")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/clear_cache/{password}")]
async fn clear_cache(path: web::Path<String>, data: web::Data<State>) -> impl Responder {
    if path.into_inner() == std::env::var("CLEAR_CACHE_PASSWORD").unwrap() {
        let mut con = data.connection.clone();
        let _: () = redis::cmd("FLUSHALL").query_async(&mut con).await.unwrap();
        return HttpResponse::Ok().body("Cache cleared!");
    }
    HttpResponse::Ok().body("Wrong password!")
}

#[get("/avatar/{uuid}/{size}/{helm}")]
async fn get_avatar(path: web::Path<(Uuid, u32, bool)>, data: web::Data<State>) -> impl Responder {
    let identifier = path.0.to_string().split_off(30) + &path.2.to_string();
    let mut con = data.connection.clone();
    let size = path.1;

    let key: Result<String, _> = get(&identifier, &mut con).await;
    match key {
        Ok(key) => {
            let mut buffer = key
                .split(",")
                .map(|x| x.parse::<u8>().unwrap())
                .collect::<Vec<u8>>();

            let avatar = image::load_from_memory(&buffer).unwrap().to_rgba8();

            if size != 8 {
                // resize the 8px avatar to the proper size
                let avatar = resize(&avatar, size);
                buffer = encode_png(avatar, size, image::ColorType::Rgb8);
            }

            return HttpResponse::build(StatusCode::OK)
                .content_type("image/png")
                .body(buffer);
        }
        Err(_) => {}
    }

    let skin = get_skin_bytes(path.0).await;
    let mut avatar = crop(skin.clone(), 8, 8, 8, 8);

    if path.2 == true {
        let helm = crop(skin, 40, 8, 8, 8);
        imageops::overlay(&mut avatar, &helm, 0, 0);
    }

    let mut png_buffer: Vec<u8> = encode_png(avatar.clone(), 8, image::ColorType::Rgb8);
    // If avatar is not 8px wide
    if avatar.width() != size {
        // Cache the 8px avatar
        let avatar_str = png_buffer.to_vec();
        let avatar_str = avatar_str
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");

        set(identifier.clone(), avatar_str, &mut con).await;

        // Now resize it to the proper size
        avatar = resize(&avatar, size);
    } else {
        return HttpResponse::build(StatusCode::OK)
            .content_type(header::ContentType("image/png".parse().unwrap()))
            .insert_header(("Cache-Control", "max-age=1200"))
            .body(png_buffer);
    }

    png_buffer = encode_png(avatar, size, image::ColorType::Rgb8);

    HttpResponse::build(StatusCode::OK)
        .content_type(header::ContentType("image/png".parse().unwrap()))
        .insert_header(("Cache-Control", "max-age=1200"))
        .body(png_buffer)
}

#[get("/skin/{uuid}/{size}")]
async fn get_skin(path: web::Path<(Uuid, u32)>) -> impl Responder {
    let size = path.1;
    let skin = get_skin_bytes(path.0).await;
    let mut skin = image::load_from_memory(&skin).unwrap().to_rgba8();

    if size != 64 {
        skin = imageops::resize(&skin, size, size, imageops::FilterType::Nearest);
    }

    let png_buffer = encode_png(skin, size, image::ColorType::Rgba8);
    HttpResponse::build(StatusCode::OK)
        .content_type(header::ContentType("image/png".parse().unwrap()))
        .insert_header(("Cache-Control", "max-age=1200"))
        .body(png_buffer)
}

#[get("/skin/{uuid}")]
async fn get_skin_64(path: web::Path<Uuid>) -> impl Responder {
    let skin = get_skin_bytes(path.into_inner()).await;
    let skin = image::load_from_memory(&skin).unwrap().to_rgba8();

    let png_buffer = encode_png(skin, 64, image::ColorType::Rgba8);
    HttpResponse::build(StatusCode::OK)
        .content_type(header::ContentType("image/png".parse().unwrap()))
        .insert_header(("Cache-Control", "max-age=1200"))
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

    let port = std::env::var("PORT").expect("Missing port number");
    let port = port.parse::<u16>().expect("Port is not a number");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(State {
                connection: con.clone(),
            }))
            .service(health)
            .service(clear_cache)
            .service(get_avatar)
            .service(get_skin)
            .service(get_skin_64)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
