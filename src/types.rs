use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub enum UuidOrString {
    Uuid(Uuid),
    String(String),
}

impl std::str::FromStr for UuidOrString {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Uuid::parse_str(s) {
            Ok(uuid) => Ok(UuidOrString::Uuid(uuid)),
            Err(_) => Ok(UuidOrString::String(s.to_string())),
        }
    }
}

impl Clone for UuidOrString {
    fn clone(&self) -> Self {
        match self {
            UuidOrString::Uuid(uuid) => UuidOrString::Uuid(*uuid),
            UuidOrString::String(string) => UuidOrString::String(string.clone()),
        }
    }
}

pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl IsEmpty for Vec<u8> {
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct State {
    pub connection: MultiplexedConnection,
    pub clear_cache_password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MojangProfile {
    pub id: String,
    pub name: String,
    pub properties: Option<Vec<Property>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedProperty {
    pub timestamp: i64,
    pub profile_id: String,
    pub profile_name: String,
    pub textures: Textures,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Textures {
    #[serde(rename = "SKIN")]
    pub skin: Skin,
    #[serde(rename = "CAPE")]
    pub cape: Option<Cape>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub url: String,
    pub metadata: Option<Metadata>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub model: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cape {
    pub url: String,
}
