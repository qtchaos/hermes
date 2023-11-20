use crate::{
    types::State,
    utils::{crop, encode_png, get, get_id, get_skin_bytes, resize, set},
};
use actix_web::{
    get,
    http::header::{self},
    web, App, HttpResponse, HttpServer, Responder,
};
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
    let uuid = path.0;
    let size = path.1;
    let helm = path.2;
    let identifier = get_id(uuid, helm);
    let mut con = data.connection.clone();
    let mut response = HttpResponse::build(StatusCode::OK);
    response.append_header((header::CONTENT_TYPE, "image/png"));
    response.append_header((header::CACHE_CONTROL, "max-age=1200"));
    response.append_header((header::SERVER, "Ziria"));

    /*
      STEP: Size validation.
    */
    if size > 512 || size < 8 || size % 8 != 0 {
        return HttpResponse::build(StatusCode::BAD_REQUEST)
            .body("Size must be between 8 and 512, and divisible by 8.");
    }

    /*
      STEP: Cache loading.
    */
    let key: Result<Vec<u8>, _> = get(&identifier, &mut con).await;
    match key {
        Ok(mut buffer) => {
            const DATA: [u8; 36] = [
                // Offset 0x00000000 to 0x00000023
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
                0x44, 0x52, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08, 0x08, 0x02, 0x00, 0x00,
                0x00, 0x4B, 0x6D, 0x29, 0xDC, 0x00, 0x00, 0x00,
            ];

            buffer.splice(0..0, DATA.iter().cloned());
            let avatar = image::load_from_memory(&buffer).unwrap().to_rgba8();

            // If the avatar is greater than 8px, resize the cached avatar
            if size > 8 {
                let avatar = resize(&avatar, size);
                buffer = encode_png(avatar, size, image::ColorType::Rgb8);
            }

            return response.body(buffer);
        }
        Err(_) => {}
    }

    let skin = match get_skin_bytes(path.0).await {
        Ok(skin) => skin,
        Err(_) => {
            return HttpResponse::build(StatusCode::NOT_FOUND).body("Skin not found!");
        }
    };

    let mut avatar = crop(skin.clone(), 8, 8, 8, 8);

    if helm == true {
        let helm = crop(skin, 40, 8, 8, 8);
        imageops::overlay(&mut avatar, &helm, 0, 0);
    }

    let buffer: Vec<u8> = encode_png(avatar.clone(), 8, image::ColorType::Rgb8);

    /*
     STEP: Creating cache.
    */
    let avatar_bytes = buffer.to_vec();
    let avatar_bytes = &avatar_bytes[36..];
    set(identifier.clone(), avatar_bytes, &mut con).await;

    if avatar.width() != size {
        // Now resize it to the proper size to return it
        avatar = resize(&avatar, size);
    } else {
        return response.body(buffer);
    }

    response.body(encode_png(avatar, size, image::ColorType::Rgb8))
}

#[get("/skin/{uuid}/{size}")]
async fn get_skin(path: web::Path<(Uuid, u32)>) -> impl Responder {
    let size = path.1;
    let skin = match get_skin_bytes(path.0).await {
        Ok(skin) => skin,
        Err(_) => {
            return HttpResponse::build(StatusCode::NOT_FOUND).body("Skin not found!");
        }
    };

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
    let skin = match get_skin_bytes(path.into_inner()).await {
        Ok(skin) => skin,
        Err(_) => {
            return HttpResponse::build(StatusCode::NOT_FOUND).body("Skin not found!");
        }
    };

    let skin = image::load_from_memory(&skin).unwrap().to_rgba8();

    let buffer = encode_png(skin, 64, image::ColorType::Rgba8);
    HttpResponse::build(StatusCode::OK)
        .content_type(header::ContentType("image/png".parse().unwrap()))
        .insert_header(("Cache-Control", "max-age=1200"))
        .body(buffer)
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
