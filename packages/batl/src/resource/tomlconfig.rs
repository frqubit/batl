#![allow(clippy::exhaustive_structs)]
#![allow(clippy::exhaustive_enums)]

use batl_macros::{environment_struct_impl, versioned_identical};
use crate::error::EyreResult;
use crate::resource::Name;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};

versioned_identical!("0.3.0" => "latest" : [
	Environment,
	Repository,
	Workspace,
	Scripts,
	Dependencies,
	Restrict,
	Restrictor
]);

versioned_identical!("0.2.2" => "0.3.0" : [
	Repository,
	Workspace,
	Scripts,
	Dependencies,
	Restrict,
	Restrictor,
	Links
]);

environment_struct_impl!("0.2.0");
environment_struct_impl!("0.2.1");
environment_struct_impl!("0.2.2");
environment_struct_impl!("0.3.0");

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Repository0_2_2 {
	pub name: Name,
	pub version: semver::Version,
	pub git: Option<RepositoryGit0_2_2>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Workspace0_2_2 {
	pub name: Name,
	pub version: semver::Version
}

pub type Links0_2_2 = Links0_2_1;
pub type RepositoryGit0_2_2 = RepositoryGit0_2_1;
pub type Scripts0_2_2 = Scripts0_2_1;
pub type Dependencies0_2_2 = Dependencies0_2_1;
pub type Restrict0_2_2 = HashMap<Restrictor0_2_2, RestrictorSettings0_2_2>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Restrictor0_2_2 {
	Windows,
	Linux,
	Unix,
	MacOs
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RestrictorSettings0_2_2 {
	pub include: Option<RestrictRequirement0_2_2>,
	pub dependencies: Option<Dependencies0_2_2>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RestrictRequirement0_2_2 {
	Deny,
	Allow,
	Require
}

pub type Repository0_2_1 = Repository0_2_0;
pub type Workspace0_2_1 = Workspace0_2_0;
pub type Links0_2_1 = Links0_2_0;
pub type RepositoryGit0_2_1 = RepositoryGit0_2_0;
pub type Scripts0_2_1 = Scripts0_2_0;
pub type Dependencies0_2_1 = Dependencies0_2_0;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Repository0_2_0 {
	pub name: Name,
	pub version: semver::Version,
	pub build: Option<String>,
	pub git: Option<RepositoryGit0_2_0>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Workspace0_2_0 {
	pub name: Name,
	pub version: semver::Version,
	pub build: Option<String>
}

pub type Links0_2_0 = HashMap<String, Name>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RepositoryGit0_2_0 {
	pub url: String,
	pub path: String
}

pub type Scripts0_2_0 = HashMap<String, String>;
pub type Dependencies0_2_0 = HashMap<Name, String>;


/// Writes a toml struct to a path
/// 
/// # Errors
/// 
/// Propogates any IO errors received while writing the file.
#[inline]
pub fn write_toml<T: serde::Serialize>(path: &Path, data: &T) -> Result<(), std::io::Error> {
	let mut file = std::fs::File::create(path)?;

	file.write_all(toml::to_string(data).unwrap_or_default().as_bytes())?;

	Ok(())
}

/// Returns `None` if a hashmap is empty
#[inline]
#[must_use]
pub fn hashmap_to_option_hashmap<K, V, S>(map: HashMap<K, V, S>) -> Option<HashMap<K, V, S>> {
	if map.is_empty() {
		None
	} else {
		Some(map)
	}
}

pub trait TomlConfig: Sized {
	/// Reads a toml file from a path
	/// 
	/// # Errors
	/// 
	/// Propogates any toml and IO errors to the caller
	fn read_toml(path: &Path) -> EyreResult<Self>;

	/// Starts at the path given, then searches all
	/// parents for a batl.toml, returning the first
	/// directory with one.
	#[inline]
	#[must_use]
	fn locate_possible(current: &Path) -> Option<PathBuf> {
		let mut current_path = current.to_path_buf();

		loop {
			let batl_toml = current_path.join("batl.toml");
	
			if batl_toml.exists() {
				break Some(current_path);
			}
	
			if !current_path.pop() {
				break None;
			}
		}
	}

	/// Starts at the given path, then searches all
	/// paths for a batl.toml *that satisfies parsing*.
	/// Returns the path of the batl.toml, including
	/// the filename.
	/// 
	/// Returns `None` if it can't be found.
	#[inline]
	#[must_use]
	fn locate(current: &Path) -> Option<PathBuf> {
		let mut search_dir = Self::locate_possible(current);

		while let Some(config_dir) = search_dir {
			let config = Self::read_toml(&config_dir.join("batl.toml"));

			match config {
				Ok(_) => {
					return Some(config_dir.join("batl.toml"))
				},
				Err(_) => {
					search_dir = config_dir
						.parent()
						.and_then(|p| Self::locate_possible(p));
				}
			}
		}

		None
	}

	/// Starts at the given path, then searches all
	/// paths for a batl.toml *that satisfies parsing*.
	/// Returns the path of the batl.toml, including
	/// the filename.
	/// 
	/// Returns `None` if it can't be found.
	#[inline]
	#[must_use]
	fn load(current: &Path) -> Option<Self> {
		let mut search_dir = Self::locate_possible(current);

		while let Some(config_dir) = search_dir {
			let config = Self::read_toml(&config_dir.join("batl.toml"));

			match config {
				Ok(config_out) => {
					return Some(config_out)
				},
				Err(_) => {
					search_dir = config_dir
						.parent()
						.and_then(|p| Self::locate_possible(p));
				}
			}
		}

		None
	}
}

#[expect(clippy::missing_trait_methods)]
impl<T> TomlConfig for T
where
	T: serde::de::DeserializeOwned
{
	#[inline]
	fn read_toml(path: &Path) -> EyreResult<Self> {
		let config_str = std::fs::read_to_string(path)?;
		Ok(toml::from_str(&config_str)?)
	}
}
