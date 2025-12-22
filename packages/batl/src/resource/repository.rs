use crate::error as batlerror;
use semver::Version;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use super::{tomlconfig, Name, Resource};
use super::archive::Archive;
use super::restrict::{Condition, Settings as RestrictSettings};
use super::tomlconfig::TomlConfig;


pub struct Repository {
	/// The actual path of the repository, absolute by standard
	path: PathBuf,

	/// The repository configuration
	config: Config,

	/// The repository name
	name: Name
}

#[derive(Default)]
#[non_exhaustive]
pub struct CreateRepositoryOptions {
	pub git: Option<tomlconfig::RepositoryGit0_2_2>
}

impl CreateRepositoryOptions {
	#[inline]
	#[must_use]
	pub const fn git(git: tomlconfig::RepositoryGit0_2_2) -> Self {
		Self {
			git: Some(git)
		}
	}
}

impl Repository {
	/// Loads the repository at the given name
	/// 
	/// # Errors
	/// 
	/// Propogates any errors found along the way
	/// Returns `None` if no repository is found.
	#[inline]
	pub fn load(name: Name) -> Result<Option<Self>, batlerror::GeneralResourceError> {
		let repo_path = crate::system::repository_root()
			.map(|p| p.join(PathBuf::from(&name)));

		if let Some(path) = repo_path {
			let toml = AnyTomlConfig::read_toml(&path.join("batl.toml"))?;
			let latest = TomlConfigLatest::from(toml);

			Ok(Some(Self {
				path,
				config: Config::from(latest),
				name
			}))
		} else {
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
	pub fn create(name: Name, options: CreateRepositoryOptions) -> Result<Self, batlerror::CreateResourceError> {
		let repo_path = crate::system::repository_root()
			.ok_or(batlerror::CreateResourceError::NotSetup)?
			.join(PathBuf::from(&name));

		if repo_path.exists() {
			return Err(batlerror::CreateResourceError::AlreadyExists);
		}

		std::fs::create_dir_all(&repo_path)?;

		let mut scripts = HashMap::new();
		scripts.insert("build".to_owned(), "echo \"No build targets\" && exit 1".to_owned());

		let mut restrictions = HashMap::new();

		#[cfg(unix)]
		let restrictor = tomlconfig::RestrictorLatest::Unix;

		#[cfg(target_os = "windows")]
		let restrictor = tomlconfig::RestrictorLatest::Windows;

		restrictions.insert(restrictor, tomlconfig::RestrictorSettings0_2_2 {
			include: Some(tomlconfig::RestrictRequirement0_2_2::Require),
			dependencies: None
		});

		let toml = TomlConfigLatest {
			environment: tomlconfig::EnvironmentLatest::default(),
			repository: tomlconfig::RepositoryLatest {
				name: name.clone(),
				version: semver::Version::new(0, 1, 0),
				git: options.git
			},
			scripts: Some(scripts),
			dependencies: None,
			restrict: Some(restrictions)
		};

		tomlconfig::write_toml(&repo_path.join("batl.toml"), &toml)?;

		Ok(Self {
			path: repo_path,
			config: toml.into(),
			name
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
	pub fn from_path(path: &Path) -> Result<Self, batlerror::GeneralResourceError> {
		let toml = AnyTomlConfig::read_toml(&path.join("batl.toml"))?;
		let latest = TomlConfigLatest::from(toml);

		Ok(Self {
			name: path.into(),
			path: path.to_path_buf(),
			config: Config::from(latest)
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
	pub fn locate_then_load(path: &Path) -> Result<Option<Self>, batlerror::GeneralResourceError> {
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
	pub fn destroy(self) -> Result<(), batlerror::DeleteResourceError> {
		std::fs::remove_dir_all(self.path())?;

		Ok(())
	}

	/// Creates an archive, this is deprecated
	/// 
	/// # Errors
	/// 
	/// Propogates any errors found along the way
	#[inline]
	pub fn archive_gen(&self) -> Result<Archive, batlerror::CreateDependentResourceError> {
		let mut walk_builder = ignore::WalkBuilder::new(self.path());

		if let Some(git) = self.config().git.clone() {
			walk_builder.add_ignore(git.path);
		}

		walk_builder.add_custom_ignore_filename("batl.ignore");

		let walk = walk_builder.build();

		let tar_path = crate::system::archive_root()
			.ok_or(batlerror::CreateResourceError::NotSetup)?
			.join("repositories")
			.join(format!("{}.tar", self.name.to_string().replace('.', "/")));

		if let Some(tar_parent) = tar_path.parent() {
			std::fs::create_dir_all(tar_parent)?;
		}

		let mut archive = tar::Builder::new(std::fs::File::create(&tar_path)?);

		for result in walk {
			let entry = result.map_err(|_err| batlerror::GeneralResourceError::Invalid)?;

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
			path: tar_path
		})
	}

	/// Get the archive for this repository
	/// 
	/// Returns `None` if it has not been generated
	#[inline]
	#[must_use]
	pub fn archive(&self) -> Option<Archive> {
		Archive::load(&self.name).ok().flatten()
	}
}

impl Resource for Repository {
	type Config = Config;

	#[inline]
	fn path(&self) -> &Path {
		&self.path
	}

	#[inline]
	fn name(&self) -> &Name {
		&self.name
	}

	#[inline]
	fn config(&self) -> &Config {
		&self.config
	}
}

#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
	pub name: Name,
	pub version: Version,
	pub git: Option<GitConfig>,
	pub scripts: HashMap<String, String>,
	pub dependencies: HashMap<Name, String>,
	pub restrict: HashMap<Condition, RestrictSettings>
}

#[derive(Clone)]
#[non_exhaustive]
pub struct GitConfig {
	pub url: String,
	pub path: String
}

#[non_exhaustive]
pub enum AnyTomlConfig {
	V0_3_0(TomlConfig0_3_0),
	V0_2_2(TomlConfig0_2_2),
	V0_2_1(TomlConfig0_2_1),
	V0_2_0(TomlConfig0_2_0)
}

#[expect(clippy::missing_trait_methods)]
impl TomlConfig for AnyTomlConfig {
	#[inline]
	fn read_toml(path: &Path) -> Result<Self, batlerror::ReadConfigError> {
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
			AnyTomlConfig::V0_3_0(v030) => v030
		}
	}
}

// CONFIG VERSIONS //
pub type TomlConfigLatest = TomlConfig0_3_0;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct TomlConfig0_3_0 {
	pub environment: tomlconfig::Environment0_3_0,
	pub repository: tomlconfig::Repository0_3_0,
	pub scripts: Option<tomlconfig::Scripts0_3_0>,
	pub dependencies: Option<tomlconfig::Dependencies0_3_0>,
	pub restrict: Option<tomlconfig::Restrict0_3_0>
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[non_exhaustive]
pub struct TomlConfig0_2_2 {
	pub environment: tomlconfig::Environment0_2_2,
	pub repository: tomlconfig::Repository0_2_2,
	pub scripts: Option<tomlconfig::Scripts0_2_2>,
	pub dependencies: Option<tomlconfig::Dependencies0_2_2>,
	pub restrict: Option<tomlconfig::Restrict0_2_2>
}

impl From<TomlConfig0_2_2> for TomlConfigLatest {
	#[inline]
	fn from(value: TomlConfig0_2_2) -> Self {
		Self {
			environment: tomlconfig::EnvironmentLatest::default(),
			repository: value.repository,
			scripts: value.scripts,
			dependencies: value.dependencies,
			restrict: value.restrict
		}
	}
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[non_exhaustive]
pub struct TomlConfig0_2_1 {
	pub environment: tomlconfig::Environment0_2_1,
	pub repository: tomlconfig::Repository0_2_1,
	pub scripts: Option<tomlconfig::Scripts0_2_1>,
	pub dependencies: Option<tomlconfig::Dependencies0_2_1>
}

impl From<TomlConfig0_2_1> for TomlConfigLatest {
	#[inline]
	fn from(value: TomlConfig0_2_1) -> Self {
		Self {
			environment: tomlconfig::EnvironmentLatest::default(),
			repository: tomlconfig::RepositoryLatest {
				name: value.repository.name,
				version: value.repository.version,
				git: value.repository.git
			},
			scripts: value.scripts,
			dependencies: value.dependencies,
			restrict: None
		}
	}
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[non_exhaustive]
pub struct TomlConfig0_2_0 {
	pub environment: tomlconfig::Environment0_2_0,
	pub repository: tomlconfig::Repository0_2_0,
	pub scripts: Option<tomlconfig::Scripts0_2_0>,
	pub dependencies: Option<tomlconfig::Dependencies0_2_0>
}

impl From<TomlConfig0_2_0> for TomlConfigLatest {
	#[inline]
	fn from(value: TomlConfig0_2_0) -> Self {
		Self {
			environment: tomlconfig::EnvironmentLatest::default(),
			repository: tomlconfig::RepositoryLatest {
				name: value.repository.name,
				version: value.repository.version,
				git: value.repository.git
			},
			scripts: value.scripts,
			dependencies: value.dependencies,
			restrict: None
		}
	}
}

impl From<TomlConfigLatest> for Config {
	#[inline]
	fn from(value: TomlConfigLatest) -> Self {
		let git = value.repository.git.map(|toml| GitConfig {
			url: toml.url,
			path: toml.path
		});

		let restrict = value.restrict
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
			restrict
		}
	}
}

impl From<Config> for TomlConfigLatest {
	#[inline]
	fn from(value: Config) -> Self {
		let git = value.git.map(|conf| tomlconfig::RepositoryGit0_2_2 {
			url: conf.url,
			path: conf.path
		});

		let restrict = value.restrict.into_iter()
			.map(|(k, v)| (k.into(), v.into()))
			.collect::<HashMap<_, _>>();

		Self {
			environment: tomlconfig::EnvironmentLatest::default(),
			repository: tomlconfig::RepositoryLatest {
				name: value.name,
				version: value.version,
				git
			},
			scripts: tomlconfig::hashmap_to_option_hashmap(value.scripts),
			dependencies: tomlconfig::hashmap_to_option_hashmap(value.dependencies),
			restrict: tomlconfig::hashmap_to_option_hashmap(restrict)
		}
	}
}
