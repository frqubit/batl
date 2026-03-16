use crate::{
    error::err_resource_does_not_exist,
    output::warn,
    resource::{
        source::RepositorySource,
        tomlconfig::{self, Environment0_3_0},
    },
    EyreResult,
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
        tracked: Option<HashMap<(Name, Version), (HashId, Vec<RepositorySource>)>>,
    ) -> EyreResult<HashMap<(Name, Version), (HashId, Vec<RepositorySource>)>> {
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

                let hash_id = dep_repo.gen_hashid()?;
                let sources = dep_repo.config().sources.clone();

                out.insert(dep_clone, (hash_id, sources));
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
                    hashid: v.0,
                    sources: v.1.into_iter().map(|item| item.into()).collect(),
                })
                .sorted()
                .collect(),
        ))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct SummarizedDependency {
    pub name: Name,
    pub version: Version,
    pub hashid: HashId,
    pub sources: tomlconfig::SourcesLatest,
}

impl Eq for SummarizedDependency {}

impl Ord for SummarizedDependency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // It's nonsensical to be equal but it's nonsensical for this to fail anyway so yeaa
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for SummarizedDependency {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_name = self.name.to_string();
        let other_name = other.name.to_string();

        if self_name > other_name {
            return Some(std::cmp::Ordering::Greater);
        } else if self_name < other_name {
            return Some(std::cmp::Ordering::Less);
        } else if self.version > other.version {
            return Some(std::cmp::Ordering::Greater);
        } else if self.version < other.version {
            return Some(std::cmp::Ordering::Less);
        }

        // Versions can technically have builds/tags that aren't counted by Ord
        // Check these with to_string
        let self_version = self.version.to_string();
        let other_version = other.version.to_string();
        if self_version > other_version {
            return Some(std::cmp::Ordering::Greater);
        } else if self_version < other_version {
            return Some(std::cmp::Ordering::Less);
        } else {
            // They're somehow the same dependency which literally makes no sense?
            warn("battalion has detected the exact same dependency registered twice in your lockfile. this is unintentional. please report.");
            return Some(std::cmp::Ordering::Equal);
        }
    }
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
