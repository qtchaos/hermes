use crate::types::UuidOrString;
use redis::AsyncCommands;

use crate::types::IsEmpty;

pub async fn set<T: redis::ToRedisArgs + Send + Sync>(
    k: String,
    v: T,
    con: &mut redis::aio::MultiplexedConnection,
) {
    let _: () = con.set(&k, v).await.unwrap();
    let _: () = con.expire(k, 1200).await.unwrap();
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
