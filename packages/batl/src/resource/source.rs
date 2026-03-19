use tempfile::TempDir;

use crate::action::batlconstant::BatlConstantTargetConfig;
use crate::action::{ActionEnv, PullAction};
use crate::error::{err_resource_does_not_exist, err_theoretical};
use crate::resource::{tomlconfig, Name, Repository};
use crate::EyreResult;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct RepositorySource {
    pub handler: Name,
    pub attrs: HashMap<String, String>,
}

impl RepositorySource {
    pub fn pull(&self) -> EyreResult<(BatlConstantTargetConfig, ActionEnv)> {
        let repository = Repository::load(self.handler.clone())?
            .ok_or(err_resource_does_not_exist(&self.handler.to_string()))?;

        let action = PullAction {
            source: self.clone(),
        };

        crate::action::run_action(&repository, action)
    }
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
