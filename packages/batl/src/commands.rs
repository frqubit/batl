use batl::resource::{self as batlres, BatlRc};
use batl::resource::tomlconfig::{TomlConfig, write_toml};
use crate::output::success;
use crate::utils::UtilityError;
use std::collections::HashMap;
use std::env::current_dir;

pub mod workspace;
pub mod link;
pub mod repository;


pub fn cmd_setup() -> Result<(), UtilityError> {
	#[cfg(target_os = "windows")]
	crate::utils::windows_symlink_perms()?;

	if batl::system::batl_root().is_some() {
		return Err(UtilityError::AlreadySetup);
	}

	let batl_root = dirs::home_dir()
		.ok_or(UtilityError::ResourceDoesNotExist("Home directory".to_string()))?
		.join("battalion");

	std::fs::create_dir_all(batl_root.join("workspaces"))?;
	std::fs::create_dir_all(batl_root.join("repositories"))?;

	let batlrc = BatlRc::default();
	
	write_toml(&batl_root.join(".batlrc"), &batlrc)?;

	println!("Battalion root directory created at {}", batl_root.display());

	Ok(())  
}

pub fn cmd_add(name: String) -> Result<(), UtilityError> {
	let config_path = batlres::repository::TomlConfigLatest::locate(&current_dir()?)
		.ok_or(UtilityError::ResourceDoesNotExist("Batallion config".to_string()))?;

	let mut config = batlres::repository::TomlConfigLatest::read_toml(&config_path)
		.map_err(|_| UtilityError::InvalidConfig)?;

	if let Some(mut deps) = config.dependencies {
		deps.insert(name.as_str().into(), "latest".to_string());

		config.dependencies = Some(deps);
	} else {
		let mut deps = HashMap::new();
		deps.insert(name.as_str().into(), "latest".to_string());

		config.dependencies = Some(deps);
	}

	write_toml(&config_path, &config)?;

	success(&format!("Added dependency {name}"));

	Ok(())
}

pub fn cmd_remove(name: String) -> Result<(), UtilityError> {
	let config_path = batlres::repository::TomlConfigLatest::locate(&current_dir()?)
		.ok_or(UtilityError::ResourceDoesNotExist("Batallion config".to_string()))?;

	let mut config = batlres::repository::TomlConfigLatest::read_toml(&config_path)
		.map_err(|_| UtilityError::InvalidConfig)?;

	if let Some(mut deps) = config.dependencies {
		if deps.remove(&name.as_str().into()).is_none() {
			return Err(UtilityError::ResourceDoesNotExist("Dependency".to_string()))
		}

		config.dependencies = Some(deps);
	} else {
		return Err(UtilityError::ResourceDoesNotExist("Dependency".to_string()));
	}

	write_toml(&config_path, &config)?;

	success(&format!("Removed dependency {name}"));

	Ok(())
}

pub fn cmd_upgrade() -> Result<(), UtilityError> {
	let batl_root = batl::system::batl_root()
		.ok_or(UtilityError::ResourceDoesNotExist("Battalion root".to_string()))?;

	if !batl_root.join("gen").exists() {
		let gen_ = batl_root.join("gen");

		std::fs::create_dir(&gen_)?;
		std::fs::create_dir(gen_.join("archives"))?;
		std::fs::create_dir(gen_.join("archives/repositories"))?;
		std::fs::create_dir(gen_.join("archives/workspaces"))?;

		success("Added gen folder");
	}

	if batl::system::batlrc().is_none() {
		let batlrc = BatlRc::default();
	
		write_toml(&batl::system::batlrc_path().expect("Nonsensical already checked for root"), &batlrc)?;

		success("Added batlrc toml");
	}

	Ok(())
}

pub fn cmd_auth() -> Result<(), UtilityError> {
	let mut key_prompt = dialoguer::Input::new();

	let api_key: String = key_prompt.with_prompt("API key").interact()?;

	let mut batlrc = batl::system::batlrc()
		.ok_or(UtilityError::ResourceDoesNotExist("BatlRc".to_string()))?;

	batlrc.api.credentials = api_key;

	write_toml(&batl::system::batlrc_path().expect("Nonsensical just read batlrc"), &batlrc)?;

	success("Added new API key");

	Ok(())
}
