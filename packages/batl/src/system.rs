use crate::error::*;
use crate::resource::batlrc::{AnyBatlRc};
use std::env::var as env_var;
use std::path::PathBuf;


/// Get the battalion root path
#[inline]
#[must_use]
pub fn batl_root() -> Option<PathBuf> {
	// 1. Check BATL_ROOT environment variable
	if let Ok(batl_root) = env_var("BATL_ROOT") {
		return Some(PathBuf::from(batl_root));
	}

	// 2. Recursively descend from current directory until .batlrc is found
	if let Ok(mut current_dir) = std::env::current_dir() {
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
	if let Some(home_dir) = dirs::home_dir() {
		let batl_dir = home_dir.join("battalion");

		if batl_dir.exists() {
			return Some(batl_dir);
		}
	}

	None
}

/// Get the battalion repository root
#[inline]
#[must_use]
pub fn repository_root() -> Option<PathBuf> {
	batl_root().map(|p| p.join("repositories"))
}

/// Get the battalion generator root
#[inline]
#[must_use]
pub fn gen_root() -> Option<PathBuf> {
	batl_root().map(|p| p.join("gen"))
}

/// Get the battalion archive root
#[inline]
#[must_use]
pub fn archive_root() -> Option<PathBuf> {
	gen_root().map(|p| p.join("archives"))
}

/// Get the battalion batlrc path
#[inline]
#[must_use]
pub fn batlrc_path() -> Option<PathBuf> {
	batl_root().map(|p| p.join(".batlrc"))
}

/// Get the battalion RC config
/// 
/// # Errors
/// 
/// Raises an error if the config does not exist
/// or cannot be legally parsed
#[inline]
pub fn batlrc() -> EyreResult<Option<AnyBatlRc>> {
	AnyBatlRc::read_toml()
}
