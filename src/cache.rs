use crate::types::UuidOrString;

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
