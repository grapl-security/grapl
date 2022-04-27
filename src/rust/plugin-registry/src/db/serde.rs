use rust_proto_new::graplinc::grapl::api::plugin_registry::v1beta1::PluginType;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseSerDeError {
    #[error("UnknownEnumValue {0}")]
    UnknownEnumValue(String),
}

// While ideally this would `impl TryFrom`, PluginType is not defined in this
// crate
pub fn try_from(value: &str) -> Result<PluginType, DatabaseSerDeError> {
    match value {
        "generator" => Ok(PluginType::Generator),
        "analyzer" => Ok(PluginType::Analyzer),
        unknown => Err(DatabaseSerDeError::UnknownEnumValue(unknown.to_owned())),
    }
}
