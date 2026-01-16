use super::archive::Archive;
use super::restrict::{Condition, Settings as RestrictSettings};
use super::summary::RepositorySummary;
use super::tomlconfig::TomlConfig;
use super::{symlink_dir, tomlconfig, Name};
use crate::error::{
    err_action_impossible_while_condition, err_battalion_not_setup, err_resource_already_exists,
    err_resource_does_not_exist, err_resource_does_not_have_thing, EyreResult,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::current_dir;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct Repository {
    /// The actual path of the repository, absolute by standard
    path: PathBuf,

    /// The repository configuration
    config: Config,

    /// The repository name
    name: Name,
}

#[derive(Default)]
#[non_exhaustive]
pub struct CreateRepositoryOptions {
    pub git: Option<tomlconfig::RepositoryGit0_2_2>,
}

impl CreateRepositoryOptions {
    #[inline]
    #[must_use]
    pub const fn git(git: tomlconfig::RepositoryGit0_2_2) -> Self {
        Self { git: Some(git) }
    }
}

impl Repository {
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub const fn name(&self) -> &Name {
        &self.name
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Loads the repository at the given name
    ///
    /// # Errors
    ///
    /// Propogates any errors found along the way
    /// Returns `None` if no repository is found.
    #[inline]
    pub fn load(name: Name) -> EyreResult<Option<Self>> {
        let name_segments = name.path_segments_as_repository_name();

        if let Some(version) = &name.version {
            // If the repository has a version, try to load local copy first
            let regular_version_path =
                crate::system::repository_root().map(|p| p.join(&name_segments));

            if let Some(path) = regular_version_path {
                if path.exists() {
                    return Self::from_path(&path).map(Option::Some);
                }
            }

            // If that fails, try to find a fetched copy
            let fetched_version_path =
                crate::system::fetched_repository_root().map(|p| p.join(&name_segments));

            if let Some(path) = fetched_version_path {
                if path.exists() {
                    return Self::from_path(&path).map(Option::Some);
                }
            }

            // If that also fails, remove the version and check if the nonversion local
            // copy matches the version requested
            let name_segments_noversion = name
                .clone()
                .without_version()
                .path_segments_as_repository_name();

            let regular_version_path =
                crate::system::repository_root().map(|p| p.join(&name_segments_noversion));

            if let Some(path) = regular_version_path {
                if path.exists() {
                    let repository = Self::from_path(&path)?;

                    if repository.config().version == version.clone() {
                        return Ok(Some(repository));
                    } else {
                        return Ok(None);
                    }
                }
            }

            Ok(None)
        } else {
            // First check for a dependency, if there's a dependency then use that
            if let Some(repository) = Repository::locate_then_load(&current_dir()?)? {
                let dependency = repository.config().dependencies.get(&name);

                if let Some(version) = dependency {
                    return Self::load(name.with_version(version.clone()));
                }
            }

            // If this fails, check for the regular local version
            let regular_path = crate::system::repository_root()
                .map(|p| p.join(name.path_segments_as_repository_name()));

            if let Some(path) = regular_path {
                if path.exists() {
                    return Self::from_path(&path).map(Option::Some);
                }
            }

            // If this fails, look for any versioned local copies and
            // choose the latest one
            let versioned_folder = crate::system::repository_root()
                .map(|p| p.join(name.path_segments_as_version_folder()));

            if let Some(path) = versioned_folder {
                if path.exists() {
                    let mut versions = std::fs::read_dir(path)?
                        .filter_map(|ent1| ent1.ok().map(|ent2| ent2.file_name()))
                        .map(|ver| Version::parse(&ver.to_string_lossy()))
                        .collect::<Result<Vec<_>, _>>()?;
                    versions.sort();

                    if let Some(version) = versions.last() {
                        let versioned_name = name.clone().with_version(version.clone());

                        // This is guaranteed to work since local versions have precedence
                        return Self::load(versioned_name);
                    }
                }
            }

            // If this fails, look for any versioned fetched copies
            // through the exact same process
            // If this fails, look for any versioned local copies and
            // choose the latest one
            let versioned_folder = crate::system::fetched_repository_root()
                .map(|p| p.join(name.path_segments_as_version_folder()));

            if let Some(path) = versioned_folder {
                if path.exists() {
                    let mut versions = std::fs::read_dir(path)?
                        .filter_map(|ent1| ent1.ok().map(|ent2| ent2.file_name()))
                        .map(|ver| Version::parse(&ver.to_string_lossy()))
                        .collect::<Result<Vec<_>, _>>()?;
                    versions.sort();

                    if let Some(version) = versions.last() {
                        let versioned_name = name.clone().with_version(version.clone());

                        // This is guaranteed to work since we know there's no local versions
                        return Self::load(versioned_name);
                    }
                }
            }

            Ok(None)
        }
    }

    /// Creates a repository at the given name, with the
    /// given options.
    ///
    /// # Errors
    ///
    /// Propogates any errors found along the way
    #[inline]
    pub fn create(name: Name, options: CreateRepositoryOptions) -> EyreResult<Self> {
        let repo_path = crate::system::repository_root()
            .ok_or(err_battalion_not_setup())?
            .join(name.path_segments_as_repository_name());

        if repo_path.exists() {
            return Err(err_resource_already_exists());
        }

        std::fs::create_dir_all(&repo_path)?;

        let mut scripts = HashMap::new();
        scripts.insert(
            "build".to_owned(),
            "echo \"No build targets\" && exit 1".to_owned(),
        );

        let mut restrictions = HashMap::new();

        #[cfg(unix)]
        let restrictor = tomlconfig::RestrictorLatest::Unix;

        #[cfg(target_os = "windows")]
        let restrictor = tomlconfig::RestrictorLatest::Windows;

        restrictions.insert(
            restrictor,
            tomlconfig::RestrictorSettings0_2_2 {
                include: Some(tomlconfig::RestrictRequirement0_2_2::Require),
                dependencies: None,
            },
        );

        let toml = TomlConfigLatest {
            environment: tomlconfig::EnvironmentLatest::default(),
            repository: tomlconfig::RepositoryLatest {
                name: name.clone(),
                version: semver::Version::new(0, 1, 0),
                git: options.git,
            },
            scripts: Some(scripts),
            dependencies: None,
            links: None,
            restrict: Some(restrictions),
        };

        tomlconfig::write_toml(&repo_path.join("batl.toml"), &toml)?;

        Ok(Self {
            path: repo_path,
            config: toml.into(),
            name,
        })
    }

    /// Saves the repository, mainly meant for lower
    /// level utilities.
    ///
    /// # Errors
    ///
    /// Propogates any errors found along the way
    #[inline]
    pub fn save(&self) -> Result<(), std::io::Error> {
        let toml = TomlConfigLatest::from(self.config.clone());

        tomlconfig::write_toml(&self.path().to_path_buf().join("batl.toml"), &toml)
    }

    /// Loads a repository from an absolute path. This
    /// is never recommended since there are no safety
    /// checks on the path, but it is available in case
    /// a situation calls for it.
    ///
    /// # Errors
    ///
    /// Propogates any errors found along the way
    #[inline]
    pub fn from_path(path: &Path) -> EyreResult<Self> {
        let name = Name::from_absolute_path(path)?;
        let toml = AnyTomlConfig::read_toml(&path.join("batl.toml"))?;
        let latest = TomlConfigLatest::from(toml);

        Ok(Self {
            name,
            path: path.to_path_buf(),
            config: Config::from(latest),
        })
    }

    /// Searches the path - along with all of its
    /// parents - for a working configuration.
    ///
    /// # Errors
    ///
    /// Propogates any errors found along the way
    /// Returns `None` if no repository is found
    #[inline]
    pub fn locate_then_load(path: &Path) -> EyreResult<Option<Self>> {
        AnyTomlConfig::locate(path)
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .map(|p| Self::from_path(&p))
            .transpose()
    }

    /// Get the scripts hashmap
    #[inline]
    #[must_use]
    pub fn scripts(&self) -> HashMap<String, String> {
        self.config.scripts.clone()
    }

    /// Get a specific script
    #[inline]
    #[must_use]
    pub fn script(&self, name: &str) -> Option<String> {
        self.scripts().get(name).cloned()
    }

    /// Destroy the repository from the filesystem, this
    /// is not reversible!
    ///
    /// # Errors
    /// Propogates any errors found along the way
    #[inline]
    pub fn destroy(self) -> EyreResult<()> {
        std::fs::remove_dir_all(self.path())?;

        Ok(())
    }

    /// Creates an archive, this is deprecated
    ///
    /// # Errors
    ///
    /// Propogates any errors found along the way
    #[inline]
    pub fn archive_gen(&self) -> EyreResult<Archive> {
        let mut walk_builder = ignore::WalkBuilder::new(self.path());

        if let Some(git) = self.config().git.clone() {
            walk_builder.add_ignore(git.path);
        }

        walk_builder.add_custom_ignore_filename("batl.ignore");

        let walk = walk_builder.build();

        let tar_path = crate::system::archive_root()
            .ok_or(err_battalion_not_setup())?
            .join("repositories")
            .join(format!("{}.tar", self.name.to_string().replace('.', "/")));

        if let Some(tar_parent) = tar_path.parent() {
            std::fs::create_dir_all(tar_parent)?;
        }

        let mut archive = tar::Builder::new(std::fs::File::create(&tar_path)?);

        for result in walk {
            let entry = result?;

            let abs_path = entry.path();

            if abs_path.is_dir() {
                continue;
            }

            let rel_path_opt = pathdiff::diff_paths(abs_path, self.path());

            if let Some(rel_path) = rel_path_opt {
                archive.append_path_with_name(abs_path, rel_path)?;
            }
        }

        let archive_file = archive.into_inner()?;

        Ok(Archive {
            tar: tar::Archive::new(archive_file),
            path: tar_path,
        })
    }

    /// Get the archive for this repository
    ///
    /// Returns `None` if it has not been generated
    #[inline]
    pub fn archive(&self) -> EyreResult<Option<Archive>> {
        // self.archive_gen()

        Archive::load(&self.name)
    }

    pub fn all_dependencies(&self) -> EyreResult<Vec<Name>> {
        let mut out: Vec<_> = self.config.dependencies.keys().cloned().collect();

        for name in out.clone() {
            let repository = Repository::load(name.clone())?
                .ok_or(err_resource_does_not_exist(&name.to_string()))?;

            out.extend(repository.all_dependencies()?);
        }

        Ok(out)
    }

    pub fn add_dependency(
        &mut self,
        name: &Name,
        version: Option<&Version>,
    ) -> EyreResult<&mut Self> {
        let version = match version {
            Some(v) => v.clone(),
            None => {
                let repository = Repository::load(name.clone())?
                    .ok_or(err_resource_does_not_exist(&name.to_string()))?;
                repository.config().version.clone()
            }
        };

        self.config.dependencies.insert(name.clone(), version);
        self.save()?;

        Ok(self)
    }

    pub fn remove_dependency(&mut self, name: &Name) -> EyreResult<&mut Self> {
        if self.config.links.contains_key(name) {
            return Err(err_action_impossible_while_condition(
                "removing dependency",
                "dependency is linked",
            ));
        }

        self.config.dependencies.remove(name);
        self.save()?;

        Ok(self)
    }

    pub fn add_link(&mut self, repository: &Repository, path: PathBuf) -> EyreResult<&mut Self> {
        let name = repository.name().clone();

        if path.exists() {
            return Err(err_resource_already_exists());
        }

        if !self.config.dependencies.contains_key(&name) {
            return Err(err_resource_does_not_have_thing(
                "repository",
                &name.to_string(),
            ));
        }

        self.add_path_to_gitignore_file(&path)?;

        symlink_dir(repository.path(), &path)?;
        self.config.links.insert(name, path);

        self.save()?;

        Ok(self)
    }

    pub fn remove_link(&mut self, name: &Name) -> EyreResult<&mut Self> {
        let link = self
            .config
            .links
            .get(&name)
            .ok_or(err_resource_does_not_have_thing(
                "repository",
                &format!("link with name {name}"),
            ))?;

        if link.exists() {
            std::fs::remove_file(link)?;
        }

        self.remove_path_from_gitignore_file(link)?;

        self.config.links.remove(name);

        self.save()?;

        Ok(self)
    }

    fn add_path_to_gitignore_file(&self, path: &Path) -> EyreResult<()> {
        let gitignore_path = self.path.join(".gitignore");
        let gitignore_content = match gitignore_path.exists() {
            false => String::new(),
            true => {
                let mut file = std::fs::File::open(&gitignore_path)?;
                let mut out = String::new();
                file.read_to_string(&mut out)?;
                out
            }
        };

        let mut gitignore_lines: Vec<String> =
            gitignore_content.split('\n').map(String::from).collect();
        let gitignore_batl_line = gitignore_lines
            .iter()
            .position(|v| *v == "# batl.gitignore begin DO NOT MODIFY");

        if let Some(gitignore_batlpos) = gitignore_batl_line {
            // Battalion is already in gitignore
            gitignore_lines.insert(gitignore_batlpos + 1, path.to_string_lossy().into());
        } else {
            gitignore_lines.push("# batl.gitignore begin DO NOT MODIFY".into());
            gitignore_lines.push(path.to_string_lossy().into());
            gitignore_lines.push("# batl.gitignore end DO NOT MODIFY".into());
        }

        let output = gitignore_lines.join("\n").into_bytes();
        let mut out_file = std::fs::File::create(gitignore_path)?;
        out_file.write(&output)?;

        Ok(())
    }

    fn remove_path_from_gitignore_file(&self, path: &Path) -> EyreResult<()> {
        let gitignore_path = self.path.join(".gitignore");
        if !gitignore_path.exists() {
            return Ok(());
        }

        let gitignore_content = {
            let mut file = std::fs::File::open(&gitignore_path)?;
            let mut out = String::new();
            file.read_to_string(&mut out)?;
            out
        };

        let mut gitignore_lines: Vec<String> =
            gitignore_content.split('\n').map(String::from).collect();
        let gitignore_batl_line = gitignore_lines
            .iter()
            .position(|v| *v == "# batl.gitignore begin DO NOT MODIFY");
        let searching = path.to_string_lossy();

        if let Some(gitignore_batl_begin) = gitignore_batl_line {
            let found = gitignore_lines
                .iter()
                .skip(gitignore_batl_begin)
                .enumerate()
                .find(|(_, v)| *v == &searching || *v == "# batl.gitignore end DO NOT MODIFY");

            if let Some((position, value)) = found {
                if value == "# batl.gitignore end DO NOT MODIFY" {
                    return Ok(());
                } else {
                    gitignore_lines.remove(position + gitignore_batl_begin);
                }
            }
        }

        let output = gitignore_lines.join("\n").into_bytes();
        let mut out_file = std::fs::File::create(gitignore_path)?;
        out_file.write(&output)?;

        Ok(())
    }

    pub fn summarize(&self) -> EyreResult<RepositorySummary> {
        RepositorySummary::of_repository(self)
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    pub name: Name,
    pub version: Version,
    pub git: Option<GitConfig>,
    pub scripts: HashMap<String, String>,
    pub dependencies: HashMap<Name, Version>,
    pub links: HashMap<Name, PathBuf>,
    pub restrict: HashMap<Condition, RestrictSettings>,
}

#[derive(Clone)]
#[non_exhaustive]
pub struct GitConfig {
    pub url: String,
    pub path: String,
}

#[non_exhaustive]
pub enum AnyTomlConfig {
    V0_3_0(TomlConfig0_3_0),
    V0_2_2(TomlConfig0_2_2),
    V0_2_1(TomlConfig0_2_1),
    V0_2_0(TomlConfig0_2_0),
}

#[expect(clippy::missing_trait_methods)]
impl TomlConfig for AnyTomlConfig {
    #[inline]
    fn read_toml(path: &Path) -> EyreResult<Self> {
        let config_str = std::fs::read_to_string(path)?;

        if let Ok(v030) = toml::from_str(&config_str) {
            return Ok(Self::V0_3_0(v030));
        }

        if let Ok(v022) = toml::from_str(&config_str) {
            return Ok(Self::V0_2_2(v022));
        }

        if let Ok(v022) = toml::from_str(&config_str) {
            return Ok(Self::V0_2_1(v022));
        }

        Ok(Self::V0_2_0(toml::from_str(&config_str)?))
    }
}

impl From<AnyTomlConfig> for TomlConfigLatest {
    #[inline]
    fn from(value: AnyTomlConfig) -> Self {
        match value {
            AnyTomlConfig::V0_2_0(v020) => v020.into(),
            AnyTomlConfig::V0_2_1(v021) => v021.into(),
            AnyTomlConfig::V0_2_2(v022) => v022.into(),
            AnyTomlConfig::V0_3_0(v030) => v030,
        }
    }
}

// CONFIG VERSIONS //
pub type TomlConfigLatest = TomlConfig0_3_0;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TomlConfig0_3_0 {
    pub environment: tomlconfig::Environment0_3_0,
    pub repository: tomlconfig::Repository0_3_0,
    pub scripts: Option<tomlconfig::Scripts0_3_0>,
    pub dependencies: Option<tomlconfig::Dependencies0_3_0>,
    pub links: Option<tomlconfig::Links0_3_0>,
    pub restrict: Option<tomlconfig::Restrict0_3_0>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[non_exhaustive]
pub struct TomlConfig0_2_2 {
    pub environment: tomlconfig::Environment0_2_2,
    pub repository: tomlconfig::Repository0_2_2,
    pub scripts: Option<tomlconfig::Scripts0_2_2>,
    pub dependencies: Option<tomlconfig::Dependencies0_2_2>,
    pub restrict: Option<tomlconfig::Restrict0_2_2>,
}

impl From<TomlConfig0_2_2> for TomlConfigLatest {
    #[inline]
    fn from(value: TomlConfig0_2_2) -> Self {
        let dependencies = value.dependencies.map(|deps| {
            deps.into_iter()
                .map(|(k, v)| {
                    let version = Version::parse(&v).unwrap_or(Version::new(0, 0, 0));
                    (k, version)
                })
                .collect()
        });

        Self {
            environment: tomlconfig::EnvironmentLatest::default(),
            repository: value.repository,
            scripts: value.scripts,
            dependencies,
            links: None,
            restrict: value.restrict,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[non_exhaustive]
pub struct TomlConfig0_2_1 {
    pub environment: tomlconfig::Environment0_2_1,
    pub repository: tomlconfig::Repository0_2_1,
    pub scripts: Option<tomlconfig::Scripts0_2_1>,
    pub dependencies: Option<tomlconfig::Dependencies0_2_1>,
}

impl From<TomlConfig0_2_1> for TomlConfigLatest {
    #[inline]
    fn from(value: TomlConfig0_2_1) -> Self {
        TomlConfig0_2_2::from(value).into()
    }
}

impl From<TomlConfig0_2_1> for TomlConfig0_2_2 {
    #[inline]
    fn from(value: TomlConfig0_2_1) -> Self {
        Self {
            environment: tomlconfig::Environment0_2_2::default(),
            repository: tomlconfig::Repository0_2_2 {
                name: value.repository.name,
                version: value.repository.version,
                git: value.repository.git,
            },
            scripts: value.scripts,
            dependencies: value.dependencies,
            restrict: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[non_exhaustive]
pub struct TomlConfig0_2_0 {
    pub environment: tomlconfig::Environment0_2_0,
    pub repository: tomlconfig::Repository0_2_0,
    pub scripts: Option<tomlconfig::Scripts0_2_0>,
    pub dependencies: Option<tomlconfig::Dependencies0_2_0>,
}

impl From<TomlConfig0_2_0> for TomlConfigLatest {
    #[inline]
    fn from(value: TomlConfig0_2_0) -> Self {
        TomlConfig0_2_2::from(value).into()
    }
}

impl From<TomlConfig0_2_0> for TomlConfig0_2_2 {
    #[inline]
    fn from(value: TomlConfig0_2_0) -> Self {
        Self {
            environment: tomlconfig::Environment0_2_2::default(),
            repository: tomlconfig::Repository0_2_2 {
                name: value.repository.name,
                version: value.repository.version,
                git: value.repository.git,
            },
            scripts: value.scripts,
            dependencies: value.dependencies,
            restrict: None,
        }
    }
}

impl From<TomlConfigLatest> for Config {
    #[inline]
    fn from(value: TomlConfigLatest) -> Self {
        let git = value.repository.git.map(|toml| GitConfig {
            url: toml.url,
            path: toml.path,
        });

        let restrict = value
            .restrict
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect::<HashMap<_, _>>();

        Self {
            name: value.repository.name,
            version: value.repository.version,
            git,
            scripts: value.scripts.unwrap_or_default(),
            dependencies: value.dependencies.unwrap_or_default(),
            links: value.links.unwrap_or_default(),
            restrict,
        }
    }
}

impl From<Config> for TomlConfigLatest {
    #[inline]
    fn from(value: Config) -> Self {
        let git = value.git.map(|conf| tomlconfig::RepositoryGit0_2_2 {
            url: conf.url,
            path: conf.path,
        });

        let restrict = value
            .restrict
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect::<HashMap<_, _>>();

        Self {
            environment: tomlconfig::EnvironmentLatest::default(),
            repository: tomlconfig::RepositoryLatest {
                name: value.name,
                version: value.version,
                git,
            },
            scripts: tomlconfig::hashmap_to_option_hashmap(value.scripts),
            dependencies: tomlconfig::hashmap_to_option_hashmap(value.dependencies),
            links: tomlconfig::hashmap_to_option_hashmap(value.links),
            restrict: tomlconfig::hashmap_to_option_hashmap(restrict),
        }
    }
}
