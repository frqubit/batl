use crate::config::*;
use crate::utils::write_toml;
use semver::Version;
use std::collections::HashMap;
use std::convert::Infallible;
use std::env::var as env_var;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;


pub struct System;

impl System {
	pub fn batl_root() -> Option<PathBuf> {
		// 1. Check BATL_ROOT environment variable
		if let Ok(batl_root) = env_var("BATL_ROOT") {
			return Some(PathBuf::from(batl_root));
		}

		// 2. Recursively descend from current directory until .batlrc is found
		if let Some(current_dir) = std::env::current_dir().ok() {
			let mut current_dir = current_dir;

			loop {
				if current_dir.join(".batlrc").exists() {
					return Some(current_dir);
				}

				if !current_dir.pop() {
					break;
				}
			}
		}

		// 3. Check for battalion folder in home directory
		if let Ok(home_dir) = env_var("HOME") {
			let batl_dir = PathBuf::from(home_dir).join("battalion");

			if batl_dir.exists() {
				return Some(batl_dir);
			}
		}

		None
	}

	pub fn workspace_root() -> Option<PathBuf> {
		Self::batl_root().map(|p| p.join("workspaces"))
	}

	pub fn repository_root() -> Option<PathBuf> {
		Self::batl_root().map(|p| p.join("repositories"))
	}
}

#[derive(Debug, Clone)]
pub struct ResourceName(Vec<String>);

impl ResourceName {
	fn new(components: Vec<String>) -> Self {
		Self(components)
	}

	fn components(&self) -> &Vec<String> {
		&self.0
	}
}

impl From<&ResourceName> for PathBuf {
	fn from(value: &ResourceName) -> Self {
		let parts = value.components();

		let mut path = PathBuf::new();

		for part in parts.iter().take(parts.len() - 1) {
			path = path.join(format!("@{}", part));
		}
		path = path.join(parts.last().unwrap());

		path
	}
}

impl From<&Path> for ResourceName {
	fn from(value: &Path) -> Self {
		let mut value = value.iter().rev();

		let mut parts = vec![value.next().expect("Nonsensical empty path").to_string_lossy().to_string()];

		while let Some(val) = value.next() {
			let val = val.to_string_lossy().to_string();

			if val.starts_with('@') {
				parts.push(val.get(1..).unwrap().to_string())
			}
		}

		ResourceName::new(parts)
	}
}

impl FromStr for ResourceName {
	type Err = Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::new(s.split('/').map(ToString::to_string).collect()))
	}
}

impl From<String> for ResourceName {
	fn from(value: String) -> Self {
		Self::from_str(&value).unwrap()
	}
}

impl From<&str> for ResourceName {
	fn from(value: &str) -> Self {
		Self::from_str(value).unwrap()
	}
}

impl Display for ResourceName {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0.join("/"))
	}
}

pub trait Resource {
	fn path(&self) -> &Path;
	fn name(&self) -> &ResourceName;
	fn config(&self) -> &Config;
}

pub struct Repository {
	path: PathBuf,
	config: Config,
	name: ResourceName
}

#[derive(Clone, Default)]
#[non_exhaustive]
pub struct CreateRepositoryOptions {
	pub git: Option<RepositoryGitConfig>
}

impl Repository {
	pub fn load(name: ResourceName) -> Result<Option<Self>, GeneralResourceError> {
		let repo_path = System::repository_root()
			.map(|p| p.join(PathBuf::from(&name)));

		if let Some(path) = repo_path {
			let config = Config::read(&path.join("batl.toml"))?;

			Ok(Some(Self {
				path,
				config,
				name
			}))
		} else {
			Ok(None)
		}
	}

	pub fn create(name: ResourceName, options: CreateRepositoryOptions) -> Result<Self, CreateResourceError> {
		let repo_path = System::repository_root()
			.ok_or(CreateResourceError::NotSetup)?
			.join(PathBuf::from(&name));

		if repo_path.exists() {
			return Err(CreateResourceError::AlreadyExists);
		}

		std::fs::create_dir_all(&repo_path)?;

		let mut scripts = HashMap::new();
		scripts.insert("build".to_string(), "echo \"No build targets\" && exit 1".to_string());
	
		let config = Config {
			environment: EnvConfig {
				version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
			},
			workspace: None,
			repository: Some(RepositoryConfig {
				name: name.to_string(),
				version: Version::new(0, 1, 0),
				build: None,
				git: options.git
			}),
			scripts: Some(scripts),
			dependencies: None
		};

		write_toml(&repo_path.join("batl.toml"), &config)?;

		Ok(Self {
			path: repo_path,
			config,
			name
		})
	}

	fn save(&self) -> Result<(), std::io::Error> {
		write_toml(self.path(), &self.config)
	}

	pub(crate) fn from_path(path: PathBuf) -> Result<Self, GeneralResourceError> {
		let config = Config::read(&path.join("batl.toml"))?;

		Ok(Self {
			name: path.as_path().into(),
			path,
			config,
		})
	}

	pub fn locate_then_load(path: &Path) -> Result<Option<Self>, GeneralResourceError> {
		Config::get_path_on_condition_from_dir(path, Config::is_repository)?.map(Self::from_path).transpose()
	}

	pub fn scripts(&self) -> HashMap<String, String> {
		self.config.scripts.clone().unwrap_or_default()
	}

	pub fn script(&self, name: &str) -> Option<String> {
		self.scripts().get(name).cloned()
	}

	pub fn destroy(self) -> Result<(), DeleteResourceError> {
		std::fs::remove_dir_all(self.path())?;

		Ok(())
	}

	pub fn workspaceify(&mut self, name: ResourceName) -> Result<Workspace, CreateDependentResourceError> {
		let config = &mut self.config;

		if !config.is_workspace() {
			config.workspace = Some(Default::default());
		}

		self.save()?;

		let workspace = Workspace::load(name)?.ok_or(GeneralResourceError::DoesNotExist)?;

		Ok(workspace)
	}

	pub fn config(&self) -> RepositoryConfig {
		self.config.repository.clone().expect("Nonsensical repository without repository config")
	}
}

impl Resource for Repository {
	fn path(&self) -> &Path {
		&self.path
	}

	fn name(&self) -> &ResourceName {
		&self.name
	}

	fn config(&self) -> &Config {
		&self.config
	}
}

pub struct Workspace {
	path: PathBuf,
	config: Config,
	name: ResourceName
}

impl Workspace {
	pub fn load(name: ResourceName) -> Result<Option<Self>, GeneralResourceError> {
		let repo_path = System::workspace_root()
			.map(|p| p.join(PathBuf::from(&name)));

		if let Some(path) = repo_path {
			let config = Config::read(&path.join("batl.toml"))?;

			Ok(Some(Self {
				path,
				config,
				name
			}))
		} else {
			Ok(None)
		}
	}

	pub fn create(name: ResourceName) -> Result<Self, CreateResourceError> {
		let path = System::workspace_root()
			.ok_or(CreateResourceError::NotSetup)?
			.join(PathBuf::from(&name));

		std::fs::create_dir_all(&path)?;

		let batl_toml_path = path.join("batl.toml");
		let config = Config {
			environment: EnvConfig {
				version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
			},
			workspace: Some(HashMap::new()),
			repository: None,
			scripts: None,
			dependencies: None
		};

		write_toml(&batl_toml_path, &config)?;

		Ok(Workspace {
			path,
			config,
			name
		})
	}

	pub fn create_from_repository(repository: &mut Repository) -> Result<Self, CreateDependentResourceError> {
		let name = repository.name().clone();

		// workspace cannot already exist
		let workspace_path = System::workspace_root()
			.ok_or(CreateResourceError::NotSetup)?
			.join(PathBuf::from(&name));

		if workspace_path.exists() {
			return Err(CreateResourceError::AlreadyExists.into());
		}

		std::fs::create_dir_all(workspace_path.parent().expect("Nonsensical no workspace parent fault"))?;
		std::os::unix::fs::symlink(repository.path(), workspace_path)?;

		let workspace = repository.workspaceify(repository.name().clone())?;

		Ok(workspace)
	}

	fn save(&self) -> Result<(), std::io::Error> {
		write_toml(self.path(), &self.config)
	}

	pub(crate) fn from_path(path: PathBuf) -> Result<Self, GeneralResourceError> {
		let config = Config::read(&path.join("batl.toml"))?;

		Ok(Self {
			name: path.as_path().into(),
			path,
			config
		})
	}

	pub fn locate_then_load(path: &Path) -> Result<Option<Self>, GeneralResourceError> {
		Config::get_path_on_condition_from_dir(path, Config::is_workspace)?.map(Self::from_path).transpose()
	}

	pub fn links(&self) -> HashMap<String, String> {
		self.config.workspace.clone().unwrap_or_default()
	}

	pub fn link(&self, name: &str) -> Option<Repository> {
		let name = self.links().get(name)?.as_str().into();

		Repository::load(name).ok().flatten()
	}

	pub fn create_link(&mut self, name: &str, repo: &Repository) -> Result<(), CreateResourceError> {
		let mut links = self.config.workspace.clone().unwrap_or_default();

		if links.contains_key(name) {
			return Err(CreateResourceError::AlreadyExists);
		}

		links.insert(name.to_string(), repo.name().to_string());
		self.config.workspace = Some(links);

		std::os::unix::fs::symlink(repo.path(), self.path.join(name))?;

		self.save()?;

		Ok(())
	}

	pub fn unlink(&mut self, name: &str) -> Result<(), DeleteResourceError> {
		let mut links = self.config.workspace.clone().unwrap_or_default();

		if !links.contains_key(name) {
			return Err(DeleteResourceError::DoesNotExist);
		}

		links.remove(name);
		self.config.workspace = Some(links);

		std::fs::remove_file(self.path.join(&name))?;

		self.save()?;

		Ok(())
	}

	pub fn destroy(self) -> Result<(), DeleteResourceError> {
		std::fs::remove_dir_all(self.path())?;

		Ok(())
	}
}

impl Resource for Workspace {
	fn path(&self) -> &Path {
		&self.path
	}

	fn name(&self) -> &ResourceName {
		&self.name
	}

	fn config(&self) -> &Config {
		&self.config
	}
}

#[derive(Debug, Error)]
pub enum CreateResourceError {
	#[error("IO Error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Battalion not set up")]
	NotSetup,
	#[error("Resource already exists")]
	AlreadyExists
}

#[derive(Debug, Error)]
pub enum CreateDependentResourceError {
	#[error("IO Error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Error while creating resource: {0}")]
	Creation(#[from] CreateResourceError),
	#[error("Error while getting dependents: {0}")]
	Dependent(#[from] GeneralResourceError)
}

#[derive(Debug, Error)]
pub enum GeneralResourceError {
	#[error("IO Error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Resource does not exist")]
	DoesNotExist,
	#[error("Resource invalid/corrupted")]
	Invalid
}

impl From<ReadConfigError> for GeneralResourceError {
	fn from(value: ReadConfigError) -> Self {
		match value {
			ReadConfigError::IoError(e) if {
				e.kind() == std::io::ErrorKind::NotFound
			} => GeneralResourceError::DoesNotExist,
			ReadConfigError::IoError(e) => e.into(),
			ReadConfigError::TomlError(_) => GeneralResourceError::Invalid
		}
	}
}

#[derive(Debug, Error)]
pub enum DeleteResourceError {
	#[error("IO Error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Resource does not exist")]
	DoesNotExist
}
