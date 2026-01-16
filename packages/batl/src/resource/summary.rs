use crate::{error::err_resource_does_not_exist, EyreResult};
use std::collections::{HashMap, HashSet};

use semver::Version;

use super::{
    restrict::{Condition, Settings as RestrictSettings},
    Name, Repository,
};

#[non_exhaustive]
pub struct RepositorySummary {
    pub name: Name,
    pub version: Version,
    pub dependencies: RecursedRepositoryDeps,
    pub restrict: HashMap<Condition, RestrictSettings>,
}

impl RepositorySummary {
    pub fn of_repository(value: &Repository) -> EyreResult<Self> {
        let deps = RecursedRepositoryDeps::of_repository(value)?;
        let config = value.config().clone();

        Ok(Self {
            name: config.name,
            version: config.version,
            dependencies: deps,
            restrict: config.restrict,
        })
    }
}

#[non_exhaustive]
pub struct RecursedRepositoryDeps(Vec<(Name, Version)>);

impl IntoIterator for RecursedRepositoryDeps {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = (Name, Version);

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl RecursedRepositoryDeps {
    fn add_deps_of_repository_to_tracked(
        repository: &Repository,
        tracked: Option<HashSet<(Name, Version)>>,
    ) -> EyreResult<HashSet<(Name, Version)>> {
        let mut out = tracked.unwrap_or_default();

        for dependency in &repository.config().dependencies {
            let dependency = (dependency.0.clone(), dependency.1.clone());

            if !out.contains(&dependency) {
                let dep_clone = dependency.clone();
                let dep_name = dependency.0.with_version(dependency.1);
                let dep_repo = Repository::load(dep_name.clone())?.ok_or(
                    err_resource_does_not_exist(&format!("dependency {dep_name}")),
                )?;

                out = Self::add_deps_of_repository_to_tracked(&dep_repo, Some(out))?;

                out.insert(dep_clone);
            }
        }

        Ok(out)
    }

    pub fn of_repository(repository: &Repository) -> EyreResult<RecursedRepositoryDeps> {
        let deps = Self::add_deps_of_repository_to_tracked(repository, None)?;

        Ok(Self(deps.into_iter().collect()))
    }
}
