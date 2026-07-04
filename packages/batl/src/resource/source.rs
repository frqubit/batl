use tempfile::TempDir;

use crate::action::batlconstant::BatlConstantTargetConfig;
use crate::action::{ActionEnv, CheckPullAction, PullAction, PushAction};
use crate::error::{err_resource_does_not_exist, err_theoretical};
use crate::resource::summary::SummarizedDependency;
use crate::resource::{tomlconfig, Name, Repository};
use crate::EyreResult;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct RepositorySource {
    pub handler: Name,
    pub attrs: HashMap<String, String>,
}

pub enum CheckPullResult {
    CannotPull,
    CanPullButDidNot,
    CanPullAndDid((BatlConstantTargetConfig, ActionEnv)),
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

    pub fn push(&self, repository: &Repository) -> EyreResult<()> {
        let handler_repo = Repository::load(self.handler.clone())?
            .ok_or(err_resource_does_not_exist(&self.handler.to_string()))?;

        let action = PushAction {
            source: self.clone(),
            repository,
        };

        crate::action::run_action(&handler_repo, action)?;

        Ok(())
    }

    pub fn check_pull(&self, dependency: &SummarizedDependency) -> EyreResult<CheckPullResult> {
        let repository = Repository::load(self.handler.clone())?
            .ok_or(err_resource_does_not_exist(&self.handler.to_string()))?;

        let action = CheckPullAction {
            source: self.clone(),
            name: dependency.name.clone(),
            version: dependency.version.clone(),
            hashid: dependency.hashid.clone(),
        };

        if crate::action::repository_can_execute_action(&repository, &action).unwrap_or(false) {
            let out = crate::action::run_action(&repository, action)?;
            if out.0 {
                return Ok(CheckPullResult::CanPullButDidNot);
            } else {
                return Ok(CheckPullResult::CannotPull);
            }
        } else {
            let action = PullAction {
                source: self.clone(),
            };

            if crate::action::repository_can_execute_action(&repository, &action).unwrap_or(false) {
                let res = crate::action::run_action(&repository, action);

                if let Ok(out) = res {
                    return Ok(CheckPullResult::CanPullAndDid(out));
                }
            }
        }

        return Ok(CheckPullResult::CannotPull);
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
