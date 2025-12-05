use crate::error as batlerror;
use semver::Version;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use super::{tomlconfig, Name, Resource};
use super::repository::Repository;
use super::tomlconfig::TomlConfig;


pub struct Workspace {
	/// The path of the workspace in the filesystem
	path: PathBuf,

	/// The configuration of the workspace
	config: Config,

	/// The name of the workspace
	name: Name
}

impl Workspace {
	/// Load the workspace with the given name from the
	/// filesystem. Returns `None` if it could not be found.
	/// 
	/// # Errors
	/// 
	/// Propogates any errors thrown during the process.
	#[inline]
	pub fn load(name: Name) -> Result<Option<Self>, batlerror::GeneralResourceError> {
		let repo_path = crate::system::workspace_root()
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

	/// Creates a workspace at the path specified.
	/// 
	/// # Errors
	/// 
	/// Propogates any errors received during creation.
	#[inline]
	pub fn create(name: Name) -> Result<Self, batlerror::CreateResourceError> {
		let path = crate::system::workspace_root()
			.ok_or(batlerror::CreateResourceError::NotSetup)?
			.join(PathBuf::from(&name));

		std::fs::create_dir_all(&path)?;

		let batl_toml_path = path.join("batl.toml");
		let toml = TomlConfigLatest {
			environment: tomlconfig::EnvironmentLatest::default(),
			workspace: tomlconfig::WorkspaceLatest {
				name: name.clone(),
				version: Version::new(0, 1, 0)
			},
			links: None,
			scripts: None,
			dependencies: None,
		};

		tomlconfig::write_toml(&batl_toml_path, &toml)?;

		Ok(Self {
			path,
			config: toml.into(),
			name
		})
	}

	/// Saves the workspace to the local filesystem
	/// 
	/// # Errors
	/// 
	/// Propogates any IO erors to the caller
	fn save(&self) -> Result<(), std::io::Error> {
		let toml = TomlConfigLatest::from(self.config.clone());

		tomlconfig::write_toml(&self.path().to_path_buf().join("batl.toml"), &toml)
	}

	/// Load a workspace from a path. This is not recommended, but is available
	/// for specific use cases. You should prefer other things like loading
	/// from a resource name, or if you need to use a path, `locate_then_load`.
	/// 
	/// # Errors
	/// 
	/// Propogates any errors back to the caller
	#[inline]
	pub fn from_path(path: &Path) -> Result<Self, batlerror::GeneralResourceError> {
		let toml = TomlConfigLatest::read_toml(&path.join("batl.toml"))?;

		Ok(Self {
			name: path.into(),
			path: path.to_path_buf(),
			config: Config::from(toml)
		})
	}

	/// Starting at the provided path, find a workspace in the
	/// parents. This can be used to find the workspace
	/// in the current directory. Returns `None` if it
	/// does not exist.
	/// 
	/// # Errors
	/// 
	/// Propogates any errors to the caller
	#[inline]
	pub fn locate_then_load(path: &Path) -> Result<Option<Self>, batlerror::GeneralResourceError> {
		TomlConfigLatest::locate(path)
			.and_then(|p| p.parent().map(Path::to_path_buf))
			.map(|p| Self::from_path(&p))
			.transpose()
	}

	/// Get the links in the workspace. Each link provides access
	/// to a repository.
	#[inline]
	#[must_use]
	pub fn links(&self) -> HashMap<String, Name> {
		self.config.links.clone()
	}

	/// Get the link with the specific name, if it
	/// exists. This also resolves the repository from
	/// the filesystem. If the repository or link does
	/// not exist, this will return `None`.
	#[inline]
	#[must_use]
	pub fn link(&self, name: &str) -> Option<Repository> {
		let res_name = self.links().get(name)?.clone();

		Repository::load(res_name).ok().flatten()
	}

	/// Given a name and repository, create a workspace
	/// link. This sets up the folders and symbolic
	/// links required to do so.
	/// 
	/// # Errors
	/// 
	/// Returns any errors received in the process.
	#[inline]
	pub fn create_link(&mut self, name: &str, repo: &Repository) -> Result<(), batlerror::CreateResourceError> {
		let mut links = self.links();

		if links.contains_key(name) {
			return Err(batlerror::CreateResourceError::AlreadyExists);
		}

		links.insert(name.to_owned(), repo.name().clone());
		self.config.links = links;

		super::symlink_dir(repo.path(), &self.path.join(name))?;

		self.save()?;

		Ok(())
	}

	/// Delete a repository link by name. This removes
	/// the entry and the symlink from the workspace.
	/// 
	/// # Errors
	/// 
	/// Propogates any errors returned to the caller.
	#[inline]
	pub fn unlink(&mut self, name: &str) -> Result<(), batlerror::DeleteResourceError> {
		let mut links = self.links();

		if !links.contains_key(name) {
			return Err(batlerror::DeleteResourceError::DoesNotExist);
		}

		links.remove(name);
		self.config.links = links;

		std::fs::remove_file(self.path.join(name))?;

		self.save()?;

		Ok(())
	}

	/// Destroy the workspace altogether. This is not reversible!
	/// 
	/// # Errors
	/// 
	/// Returns any errors back to the caller.
	#[inline]
	pub fn destroy(self) -> Result<(), batlerror::DeleteResourceError> {
		std::fs::remove_dir_all(self.path())?;

		Ok(())
	}
}

impl Resource for Workspace {
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
	pub links: HashMap<String, Name>,
	pub scripts: HashMap<String, String>,
	pub dependencies: HashMap<Name, String>
}

#[non_exhaustive]
pub enum AnyTomlConfig {
	V0_2_2(TomlConfig0_2_2),
	V0_2_1(TomlConfig0_2_1),
	V0_2_0(TomlConfig0_2_0)
}

#[expect(clippy::missing_trait_methods)]
impl TomlConfig for AnyTomlConfig {
	#[inline]
	fn read_toml(path: &Path) -> Result<Self, batlerror::ReadConfigError> {
		let config_str = std::fs::read_to_string(path)?;

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
			AnyTomlConfig::V0_2_2(v022) => v022
		}
	}
}

// CONFIG VERSIONS //
pub type TomlConfigLatest = TomlConfig0_2_2;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[non_exhaustive]
pub struct TomlConfig0_2_2 {
	pub environment: tomlconfig::Environment0_2_2,
	pub workspace: tomlconfig::Workspace0_2_2,
	pub links: Option<tomlconfig::Links0_2_2>,
	pub scripts: Option<tomlconfig::Scripts0_2_2>,
	pub dependencies: Option<tomlconfig::Dependencies0_2_2>
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[non_exhaustive]
pub struct TomlConfig0_2_1 {
	pub environment: tomlconfig::Environment0_2_1,
	pub repository: tomlconfig::Workspace0_2_1,
	pub workspace: Option<tomlconfig::Links0_2_1>,
	pub scripts: Option<tomlconfig::Scripts0_2_1>,
	pub dependencies: Option<tomlconfig::Dependencies0_2_1>
}

impl From<TomlConfig0_2_1> for TomlConfigLatest {
	#[inline]
	fn from(value: TomlConfig0_2_1) -> Self {
		Self {
			environment: tomlconfig::EnvironmentLatest::default(),
			workspace: tomlconfig::WorkspaceLatest {
				name: value.repository.name,
				version: value.repository.version
			},
			links: value.workspace,
			scripts: value.scripts,
			dependencies: value.dependencies
		}
	}
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[non_exhaustive]
pub struct TomlConfig0_2_0 {
	pub environment: tomlconfig::Environment0_2_0,
	pub repository: tomlconfig::Workspace0_2_0,
	pub workspace: Option<tomlconfig::Links0_2_0>,
	pub scripts: Option<tomlconfig::Scripts0_2_0>,
	pub dependencies: Option<tomlconfig::Dependencies0_2_0>
}

impl From<TomlConfig0_2_0> for TomlConfigLatest {
	#[inline]
	fn from(value: TomlConfig0_2_0) -> Self {
		Self {
			environment: tomlconfig::EnvironmentLatest::default(),
			workspace: tomlconfig::WorkspaceLatest {
				name: value.repository.name,
				version: value.repository.version
			},
			links: value.workspace,
			scripts: value.scripts,
			dependencies: value.dependencies
		}
	}
}

impl From<TomlConfig0_2_2> for Config {
	#[inline]
	fn from(value: TomlConfig0_2_2) -> Self {
		Self {
			name: value.workspace.name,
			version: value.workspace.version,
			links: value.links.unwrap_or_default(),
			scripts: value.scripts.unwrap_or_default(),
			dependencies: value.dependencies.unwrap_or_default()
		}
	}
}

impl From<Config> for TomlConfigLatest {
	#[inline]
	fn from(value: Config) -> Self {
		Self {
			environment: tomlconfig::EnvironmentLatest::default(),
			workspace: tomlconfig::WorkspaceLatest {
				name: value.name,
				version: value.version
			},
			links: tomlconfig::hashmap_to_option_hashmap(value.links),
			scripts: tomlconfig::hashmap_to_option_hashmap(value.scripts),
			dependencies: tomlconfig::hashmap_to_option_hashmap(value.dependencies)
		}
	}
}
