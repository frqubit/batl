use batl::resource::{Name, Repository, Resource};
use batl::resource::batlrc::AnyBatlRc;
use batl::resource::{self as batlres, BatlRc, batlrc::BatlRcLatest};
use batl::resource::tomlconfig::{TomlConfig, write_toml};
use crate::output::{error, info, success};
use crate::utils::{BATL_NAME_REGEX, UtilityError, REGISTRY_DOMAIN};
use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;
use colored::*;

pub mod repository;

pub fn cmd_ls(filter: Option<String>) -> Result<(), UtilityError> {
	let repo_root = batl::system::repository_root()
		.ok_or(UtilityError::ResourceDoesNotExist("Repository root".to_string()))?;

	let filter_path = filter.map(|v| {
		let name = Name::from(v);
		name.components().clone()
	}).unwrap_or_default().into_iter()
		.map(|v| format!("_{v}"))
		.collect::<PathBuf>();

	let search_path = repo_root.join(filter_path);

	if !search_path.exists() {
		return Ok(());
	}

	let found = std::fs::read_dir(search_path)?
		.filter_map(|entry| {
			entry.ok()
		}).map(|entry| {
			let name_os = entry.file_name();
			let name = name_os.to_string_lossy();

			if let Some(folder) = name.strip_prefix('_') {
				folder.blue()
			} else {
				name.italic()
			}
		});
	
	for name in found {
		println!("{name}");
	}

	//

	// let mut to_search: Vec<(String, PathBuf)> = std::fs::read_dir(repo_root)?
	// 	.filter_map(|entry| {
	// 		Some(("".to_string(), entry.ok()?.path()))
	// 	})
	// 	.collect();
	// let mut found: Vec<String> = Vec::new();

	// while let Some((name, path)) = to_search.pop() {
	// 	if !path.is_dir() {
	// 		continue;
	// 	}

	// 	let filename = path.file_name().unwrap().to_str().unwrap();

	// 	if let Some(filename) = filename.strip_prefix('_') {
	// 		let new_name = filename.to_string();
	// 		let new_name = format!("{name}{new_name}.");

	// 		to_search.extend(
	// 			std::fs::read_dir(path)?
	// 				.filter_map(|entry| {
	// 					Some((new_name.clone(), entry.ok()?.path()))
	// 				})
	// 		);
	// 	} else {
	// 		found.push(format!("{name}{filename}"));
	// 	}
	// }

	Ok(())
}

pub fn cmd_init(name: String) -> Result<(), UtilityError> {
	if !BATL_NAME_REGEX.is_match(&name) {
		return Err(UtilityError::InvalidName(name));
	}

	Repository::create(name.into(), Default::default())?;

	success("Initialized repository successfully");

	Ok(())
}

pub fn cmd_delete(name: String) -> Result<(), UtilityError> {
	if !BATL_NAME_REGEX.is_match(&name) {
		return Err(UtilityError::InvalidName(name));
	}

	Repository::load(name.into())?
		.ok_or(UtilityError::ResourceDoesNotExist("Repository".to_string()))?
		.destroy()?;

	success("Deleted repository successfully");

	Ok(())
}

pub fn cmd_search(name: Option<String>) -> Result<(), UtilityError> {
	let name_query = name.map(|v| format!("?q={v}")).unwrap_or("".into());
	let url = format!("{REGISTRY_DOMAIN}/pkg{name_query}");

	let body = ureq::get(&url)
		.call()?
		.body_mut()
		.read_to_string()?;

	let items: Vec<String> = serde_json::from_str(&body)?;

	for item in items {
		if let Some(repo) = item.strip_suffix(".tar") {
			println!("{}", repo.italic());
		} else {
			println!("{}", item.blue());
		}
	}

	Ok(())
}

pub fn cmd_publish(name: String) -> Result<(), UtilityError> {
	let batlrc: BatlRc = batl::system::batlrc()?
		.ok_or(UtilityError::ResourceDoesNotExist("BatlRc".to_string()))?
		.into();

	let repository = Repository::load(name.as_str().into())?
		.ok_or(UtilityError::ResourceDoesNotExist("Repository".into()))?;

	// let archive = repository.archive_gen()?;
	let archive = repository.archive()
		.ok_or(UtilityError::ResourceDoesNotExist("Archive".into()))?;

	let url = format!("{REGISTRY_DOMAIN}/pkg/{}", &repository.name().to_string().replace('.', "/"));

	let resp = ureq::post(&url)
		.header("x-api-key", &batlrc.api.credentials)
		.send(archive.to_file())?;

	if resp.status() == 200 {
		success(&format!("Published repository {name}"))
	} else {
		error(&format!("Failed to send repository: status code {}", resp.status()))
	}

	Ok(())
}

pub fn cmd_fetch(name: String) -> Result<(), UtilityError> {
	let url = format!("{REGISTRY_DOMAIN}/pkg/{}", name.to_string().replace('.', "/"));

	let resp = ureq::get(&url)
		.call()?;

	if resp.status() != 200 {
		error(&format!("Failed to fetch repository: status code {}", resp.status()));
		return Ok(())
	}

	let body = resp
		.into_body()
		.into_reader();

	let mut tar = tar::Archive::new(body);

	let repository_path = batl::system::repository_root()
		.ok_or(UtilityError::ResourceDoesNotExist("Battalion setup".to_string()))?
		.join(PathBuf::from(&Name::from(name.as_str())));

	std::fs::create_dir_all(&repository_path)?;

	tar.unpack(repository_path)?;

	success(&format!("Fetched repository {name}"));

	Ok(())
}

pub fn cmd_exec(name: Option<String>, script: String, args: Vec<String>) -> Result<(), UtilityError> {
	let repository = match &name {
		Some(val) => {
			Repository::load(val.as_str().into())?
		},
		None => Repository::locate_then_load(&current_dir()?)?
	}.ok_or(UtilityError::ResourceDoesNotExist("Repository".to_string()))?;

	let command = repository.script(&script)
		.ok_or(UtilityError::ScriptNotFound(script))?;

	info(&format!("Running script{}", name.map(|s| format!(" for link {s}")).unwrap_or("".to_string())));

	let batl_pwd = current_dir()?;

	let status = std::process::Command::new("sh")
		.current_dir(repository.path())
		.env("BATL_PWD", batl_pwd.as_os_str())
		.arg("-c")
		.arg(command)
		.arg("batl-executor")
		.args(args)
		.status()?;


	if !status.success() {
		return Err(UtilityError::ScriptError(format!("Exit code {}", status.code().unwrap_or(0))))
	}

	success("Script completed successfully");

	Ok(())
}

pub fn cmd_which(name: String) -> Result<(), UtilityError> {
	if !BATL_NAME_REGEX.is_match(&name) {
		return Err(UtilityError::InvalidName(name));
	}

	let workspace = Repository::load(name.into())?
		.ok_or(UtilityError::ResourceDoesNotExist("Workspace".into()))?;

	println!("{}", workspace.path().to_string_lossy());

	Ok(())
}

pub fn cmd_setup() -> Result<(), UtilityError> {
	#[cfg(target_os = "windows")]
	crate::utils::windows_symlink_perms()?;

	if batl::system::batl_root().is_some() {
		return Err(UtilityError::AlreadySetup);
	}

	let batl_root = dirs::home_dir()
		.ok_or(UtilityError::ResourceDoesNotExist("Home directory".to_string()))?
		.join("battalion");

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

fn migrate_at_to_underscore(path: &PathBuf) -> Result<(), UtilityError> {
	let mut to_search: Vec<PathBuf> = std::fs::read_dir(path)?
		.filter_map(|entry| {
			Some(entry.ok()?.path())
		})
		.collect();

	while let Some(path) = to_search.pop() {
		if !path.is_dir() {
			continue;
		}

		let filename = path.file_name().unwrap().to_str().unwrap();

		if filename.strip_prefix('@').is_some() {
			migrate_at_to_underscore(&path)?;
		}
	}

	let this_filename = path.file_name()
		.unwrap()
		.to_string_lossy();

	if let Some(noprefix) = this_filename.strip_prefix('@') {

		let path_parent = path.parent().unwrap().to_path_buf();
		let new_path = path_parent.join(
			format!("_{noprefix}")
		);

		std::fs::rename(path, new_path)?;
	}

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

		success("Added gen folder");
	}

	if batl::system::batlrc()?.is_none() {
		// migrate @ to _
		migrate_at_to_underscore(&batl::system::repository_root()
			.ok_or(UtilityError::ResourceDoesNotExist("Repository root".to_string()))?)?;

		let batlrc = BatlRc::default();
	
		write_toml(&batl::system::batlrc_path().expect("Nonsensical already checked for root"), &batlrc)?;

		success("Added batlrc toml");
	}

	if let Some(AnyBatlRc::V0_2_1(v021)) = batl::system::batlrc()? {
		// migrate @ to _
		migrate_at_to_underscore(&batl::system::repository_root()
			.ok_or(UtilityError::ResourceDoesNotExist("Repository root".to_string()))?)?;

		write_toml(&batl::system::batlrc_path().expect("Nonsensical already checked for root"), &BatlRcLatest::from(v021))?;

		success("migrated batlrc to 0.3.0");
	}

	Ok(())
}

pub fn cmd_auth() -> Result<(), UtilityError> {
	let key_prompt = dialoguer::Input::new();

	let api_key: String = key_prompt.with_prompt("API key").interact_text()?;

	let mut batlrc: BatlRc = batl::system::batlrc()?
		.ok_or(UtilityError::ResourceDoesNotExist("BatlRc".to_string()))?
		.into();

	batlrc.api.credentials = api_key;

	write_toml(&batl::system::batlrc_path().expect("Nonsensical just read batlrc"), &batlrc)?;

	success("Added new API key");

	Ok(())
}
