use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};

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
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MojangProfile {
    pub id: String,
    pub name: String,
    pub properties: Vec<Property>,
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
