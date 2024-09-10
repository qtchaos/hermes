use crate::types::IsEmpty;
use crate::{bytes, types::UuidOrString};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

pub async fn set<T: redis::ToRedisArgs + Send + Sync>(
    k: String,
    v: T,
    con: &mut MultiplexedConnection,
    time: i64,
) {
    let _: () = con.set(&k, v).await.unwrap();
    let _: () = con.expire(k, time).await.unwrap();
}

pub async fn get<T: redis::FromRedisValue + IsEmpty>(
    k: &String,
    con: &mut MultiplexedConnection,
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

pub fn create_id(uuid: UuidOrString, helm: bool) -> String {
    match uuid {
        UuidOrString::Uuid(uuid) => {
            let identifier = format!(
                "{}-{}",
                uuid.to_string().split_off(30),
                helm.to_string()[0..1].to_string()
            );
            return identifier;
        }
        UuidOrString::String(username) => {
            let identifier = format!(
                "{}-{}",
                username[0..username.len() - 3].to_string() + username.len().to_string().as_str(),
                helm.to_string()[0..1].to_string()
            );
            return identifier;
        }
    }
}

pub async fn set_avatar_cache(
    buffer: Vec<u8>,
    identifier: String,
    mut con: redis::aio::MultiplexedConnection,
) {
    let mut avatar_buffer = buffer.to_vec();
    avatar_buffer = bytes::strip(avatar_buffer);
    set(identifier.clone(), avatar_buffer, &mut con, 21600).await;
}
