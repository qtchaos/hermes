use crate::types::{State, UuidOrString};
use actix::Arbiter;
use actix_web::{
    get,
    http::header::{self},
    http::StatusCode,
    web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use dotenvy::dotenv;
use image::{imageops, DynamicImage::ImageRgba8};
use std::env;
use uuid::Uuid;
mod bytes;
mod cache;
mod img;
mod mojang;
mod types;

#[get("/")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/clear_cache/{password}")]
async fn clear_cache(path: web::Path<String>, data: web::Data<State>) -> impl Responder {
    if path.into_inner() == data.clear_cache_password {
        let mut con = data.connection.clone();
        let _: () = redis::cmd("FLUSHALL").query_async(&mut con).await.unwrap();
        return HttpResponse::Ok().body("Cache cleared!");
    }
    HttpResponse::Ok().body("Wrong password!")
}

#[get("/avatar/{uuid}/{size}/{helm}")]
async fn get_avatar(req: HttpRequest, data: web::Data<State>) -> impl Responder {
    let info = req.match_info();
    let uuid: UuidOrString = info.get("uuid").unwrap().parse().unwrap();
    let size: u32 = info.get("size").unwrap().parse().unwrap();
    let helm: bool = info.get("helm").unwrap().parse().unwrap();
    let identifier = cache::create_id(uuid.clone(), helm);
    let mut con = data.connection.clone();
    let mut response = HttpResponse::build(StatusCode::OK);
    response.append_header((header::CONTENT_TYPE, "image/png"));
    response.append_header((header::CACHE_CONTROL, "max-age=1200"));
    response.append_header((header::SERVER, "Ziria"));

    if size > 512 || size < 8 || size % 8 != 0 {
        return HttpResponse::build(StatusCode::BAD_REQUEST)
            .body("Size must be between 8 and 512, and divisible by 8.");
    }

    // STEP: Check if the avatar is cached, if so, load it and return it
    let key: Result<Vec<u8>, _> = cache::get(&identifier, &mut con).await;
    match key {
        Ok(mut buffer) => {
            buffer = bytes::repair(buffer);

            if size == 8 {
                return response.body(buffer);
            }

            let avatar = match image::load_from_memory(&buffer) {
                Ok(avatar) => avatar,
                Err(_) => {
                    return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Error loading avatar from cache!");
                }
            };

            let resized_avatar = img::resize(&avatar.to_rgb8(), size);
            buffer = img::encode_png(image::DynamicImage::ImageRgb8(resized_avatar));
            return response.body(buffer);
        }
        Err(_) => {}
    }

    let uuid = match uuid {
        UuidOrString::Uuid(uuid) => uuid,
        UuidOrString::String(username) => match mojang::get_uuid(username).await {
            uuid if uuid != Uuid::nil() => uuid,
            _ => {
                return HttpResponse::build(StatusCode::NOT_FOUND).body("User not found!");
            }
        },
    };

    let skin = match mojang::get_skin(uuid).await {
        Ok(skin) => skin,
        Err(_) => {
            return HttpResponse::build(StatusCode::NOT_FOUND).body("Skin not found!");
        }
    };
    let mut avatar = img::crop(skin.clone(), 8, 8, 8, 8);

    if helm {
        let helm = img::crop(skin, 40, 8, 8, 8);
        // TODO check if helm is not empty
        imageops::overlay(&mut avatar, &helm, 0, 0);
    }

    let buffer: Vec<u8> = img::encode_png(ImageRgba8(avatar.clone()));
    if avatar.width() != size {
        avatar = img::resize(&avatar, size);
    }

    // Spawn a new thread to cache the avatar
    Arbiter::spawn(
        &Arbiter::new(),
        cache::set_avatar_cache(buffer, identifier, data.connection.clone()),
    );

    response.body(img::encode_png(ImageRgba8(avatar)))
}

#[get("/skin/{uuid}/{size}")]
async fn get_skin(req: HttpRequest) -> impl Responder {
    let info = req.match_info();
    let uuid = info.get("uuid").unwrap().parse::<UuidOrString>().unwrap();
    let size = info.get("size").unwrap().parse::<u32>().unwrap();
    if size > 512 || size < 64 || size % 64 != 0 {
        return HttpResponse::build(StatusCode::BAD_REQUEST)
            .body("Size must be between 64 and 512, and divisible by 64.");
    }

    let uuid = match uuid {
        UuidOrString::Uuid(uuid) => uuid,
        UuidOrString::String(username) => match mojang::get_uuid(username).await {
            uuid if uuid != Uuid::nil() => uuid,
            _ => {
                return HttpResponse::build(StatusCode::NOT_FOUND).body("User not found!");
            }
        },
    };

    let skin = match mojang::get_skin(uuid).await {
        Ok(skin) => skin,
        Err(_) => {
            return HttpResponse::build(StatusCode::NOT_FOUND)
                .append_header((header::SERVER, "Ziria"))
                .append_header((header::CACHE_CONTROL, "max-age=1200"))
                .append_header((header::CONTENT_TYPE, "text/plain"))
                .body("Skin not found!");
        }
    };

    let mut skin = image::load_from_memory(&skin).unwrap().to_rgba8();

    if size != 64 {
        skin = img::resize(&skin, size);
    }

    let buffer = img::encode_png(ImageRgba8(skin));
    HttpResponse::build(StatusCode::OK)
        .content_type(header::ContentType("image/png".parse().unwrap()))
        .insert_header(("Cache-Control", "max-age=1200"))
        .body(buffer)
}

#[get("/skin/{uuid}")]
async fn get_skin_64(req: HttpRequest) -> impl Responder {
    let info = req.match_info();
    let uuid = info.get("uuid").unwrap().parse::<UuidOrString>().unwrap();
    let uuid = match uuid {
        UuidOrString::Uuid(uuid) => uuid,
        UuidOrString::String(username) => match mojang::get_uuid(username).await {
            uuid if uuid != Uuid::nil() => uuid,
            _ => {
                return HttpResponse::build(StatusCode::NOT_FOUND).body("User not found!");
            }
        },
    };

    let skin = match mojang::get_skin(uuid).await {
        Ok(skin) => skin,
        Err(_) => {
            return HttpResponse::build(StatusCode::NOT_FOUND).body("Skin not found!");
        }
    };

    let skin = image::load_from_memory(&skin).unwrap().to_rgba8();

    let buffer = img::encode_png(ImageRgba8(skin));
    HttpResponse::build(StatusCode::OK)
        .content_type(header::ContentType("image/png".parse().unwrap()))
        .insert_header(("Cache-Control", "max-age=1200"))
        .body(buffer)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().expect(".env file not found");
    let secret = env::vars().collect::<std::collections::HashMap<String, String>>();
    let connection_string = format!(
        "{}://{}:{}@{}:{}/",
        secret.get("REDIS_SCHEME").unwrap(),
        secret.get("REDIS_USERNAME").unwrap(),
        secret.get("REDIS_PASSWORD").unwrap(),
        secret.get("REDIS_HOST").unwrap(),
        secret.get("REDIS_PORT").unwrap()
    );
    let client = redis::Client::open(connection_string).unwrap();
    let con = client.get_multiplexed_async_connection().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(State {
                connection: con.clone(),
                clear_cache_password: secret.get("CLEAR_CACHE_PASSWORD").unwrap().to_string(),
            }))
            .service(health)
            .service(clear_cache)
            .service(get_avatar)
            .service(get_skin)
            .service(get_skin_64)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
