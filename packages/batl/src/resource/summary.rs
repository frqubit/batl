use crate::{
    error::err_resource_does_not_exist, resource::tomlconfig::Environment0_3_0, EyreResult,
};
use itertools::Itertools;
use std::collections::HashMap;

use semver::Version;
use serde::{Deserialize, Serialize};

use super::{Name, Repository};

pub type HashId = String;

#[non_exhaustive]
pub struct RepositorySummary {
    pub name: Name,
    pub version: Version,
    pub dependencies: RecursedRepositoryDeps,
}

impl RepositorySummary {
    pub fn of_repository(value: &Repository) -> EyreResult<Self> {
        let deps = RecursedRepositoryDeps::of_repository(value)?;
        let config = value.config().clone();

        Ok(Self {
            name: config.name,
            version: config.version,
            dependencies: deps,
        })
    }
}

#[non_exhaustive]
#[derive(PartialEq, Clone, Eq)]
pub struct RecursedRepositoryDeps(Vec<SummarizedDependency>);

impl IntoIterator for RecursedRepositoryDeps {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = SummarizedDependency;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl RecursedRepositoryDeps {
    fn inner_vec(&self) -> Vec<SummarizedDependency> {
        self.0.clone()
    }

    fn add_deps_of_repository_to_tracked(
        repository: &Repository,
        tracked: Option<HashMap<(Name, Version), HashId>>,
    ) -> EyreResult<HashMap<(Name, Version), HashId>> {
        let mut out = tracked.unwrap_or_default();

        for dependency in &repository.config().dependencies {
            let dependency = (dependency.0.clone(), dependency.1.clone());

            if !out.keys().contains(&dependency) {
                let dep_clone = dependency.clone();
                let dep_name = dependency.0.with_version(dependency.1);
                let dep_repo = Repository::load(dep_name.clone())?.ok_or(
                    err_resource_does_not_exist(&format!("dependency {dep_name}")),
                )?;

                out = Self::add_deps_of_repository_to_tracked(&dep_repo, Some(out))?;

                out.insert(dep_clone, dep_repo.gen_hashid()?);
            }
        }

        Ok(out)
    }

    pub fn of_repository(repository: &Repository) -> EyreResult<RecursedRepositoryDeps> {
        let deps = Self::add_deps_of_repository_to_tracked(repository, None)?;

        Ok(Self(
            deps.into_iter()
                .map(|(k, v)| SummarizedDependency {
                    name: k.0,
                    version: k.1,
                    hashid: v,
                })
                .collect(),
        ))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Eq, Hash)]
pub struct SummarizedDependency {
    pub name: Name,
    pub version: Version,
    pub hashid: HashId,
}

#[non_exhaustive]
pub enum AnySummaryFile {
    V0_3_0(SummaryFile0_3_0),
}

// FILE VERSIONS //
pub type SummaryFileLatest = SummaryFile0_3_0;

impl From<RepositorySummary> for SummaryFileLatest {
    fn from(value: RepositorySummary) -> Self {
        Self {
            environment: Environment0_3_0::default(),
            name: value.name,
            version: value.version,
            dependencies: value.dependencies.inner_vec(),
        }
    }
}

impl From<SummaryFileLatest> for RepositorySummary {
    fn from(value: SummaryFileLatest) -> Self {
        Self {
            name: value.name,
            version: value.version,
            dependencies: RecursedRepositoryDeps(value.dependencies),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SummaryFile0_3_0 {
    pub environment: Environment0_3_0,
    pub name: Name,
    pub version: Version,
    pub dependencies: Vec<SummarizedDependency>,
}
