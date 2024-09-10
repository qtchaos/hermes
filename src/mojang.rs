use crate::types::{DecodedProperty, MojangProfile};
use base64::{engine::general_purpose, Engine};
use uuid::Uuid;

pub async fn get_skin(uuid: Uuid) -> Result<Vec<u8>, &'static str> {
    let mojang_url = "https://sessionserver.mojang.com/session/minecraft/profile/";
    let resp = reqwest::get(mojang_url.to_string() + &uuid.to_string())
        .await
        .unwrap();
    let resp_result: Result<MojangProfile, reqwest::Error> = resp.json().await;
    let profile = match resp_result {
        Ok(profile) => profile,
        Err(_) => {
            // TODO add long term storage for skins/uuid, i.e cold storage to bypaass Mojang API
            return Err("Error getting profile from Mojang");
        }
    };

    let mut decoded_obj = vec![];
    if let Some(properties) = profile.properties {
        decoded_obj = general_purpose::STANDARD
            .decode(properties[0].value.as_bytes())
            .unwrap();
    }
    let decoded_obj: DecodedProperty = serde_json::from_slice(&decoded_obj).unwrap();
    let skin = reqwest::get(decoded_obj.textures.skin.url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    Ok(skin.to_vec())
}

pub async fn get_uuid(username: String) -> Uuid {
    let mojang_url = "https://api.mojang.com/users/profiles/minecraft/";
    let resp = reqwest::get(mojang_url.to_string() + &username)
        .await
        .unwrap();
    let resp_result: Result<MojangProfile, reqwest::Error> = resp.json().await;
    let profile = match resp_result {
        Ok(profile) => profile,
        Err(_) => {
            return Uuid::nil();
        }
    };
    Uuid::hyphenated(Uuid::parse_str(&profile.id).unwrap()).into()
}
