use crate::resource::{tomlconfig, Name};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct RepositorySource {
    pub handler: Name,
    pub attrs: HashMap<String, String>,
}

impl From<tomlconfig::SourceLatest> for RepositorySource {
    fn from(value: tomlconfig::SourceLatest) -> Self {
        Self {
            handler: value.handler,
            attrs: value
                .extra
                .into_iter()
                .filter_map(|(k, v)| tomlconfig::toml_value_to_string(v).map(|v| (k, v)))
                .collect(),
        }
    }
}

impl From<RepositorySource> for tomlconfig::SourceLatest {
    fn from(value: RepositorySource) -> Self {
        Self {
            handler: value.handler,
            extra: value
                .attrs
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}
