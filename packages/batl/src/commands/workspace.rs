use batl::resource::{Resource, Name, Workspace};
use clap::Subcommand;
use crate::output::*;
use crate::utils::{UtilityError, BATL_NAME_REGEX};
use std::path::PathBuf;


#[derive(Subcommand)]
pub enum Commands {
	Ls {
		filter: Option<String>
	},
	Init {
		name: String
	},
	Delete {
		name: String
	},
	Which {
		name: String
	}
}

pub fn run(cmd: Commands) -> Result<(), UtilityError> {
	match cmd {
		Commands::Ls { filter } => {
			cmd_ls(filter)
		},
		Commands::Init { name } => {
			cmd_init(name)
		},
		Commands::Delete { name } => {
			cmd_delete(name)
		},
		Commands::Which { name } => {
			cmd_which(name)
		}
	}
}

fn cmd_ls(filter: Option<String>) -> Result<(), UtilityError> {
	let workspace_root = batl::system::workspace_root()
		.ok_or(UtilityError::ResourceDoesNotExist("Workspace root".to_string()))?;

	let mut to_search: Vec<(String, PathBuf)> = std::fs::read_dir(workspace_root)?
		.filter_map(|entry| {
			Some(("".to_string(), entry.ok()?.path()))
		})
		.collect();
	let mut found: Vec<String> = Vec::new();

	while let Some((name, path)) = to_search.pop() {
		if !path.is_dir() {
			continue;
		}

		let filename = path.file_name().unwrap().to_str().unwrap();

		if let Some(stripped) = filename.strip_prefix('_') {
			let new_name = format!("{name}{stripped}/");

			to_search.extend(
				std::fs::read_dir(path)?
					.filter_map(|entry| {
						Some((new_name.clone(), entry.ok()?.path()))
					})
			);
		} else {
			found.push(format!("{name}{filename}"));
		}
	}

	for name in found {
		if let Some(filter_str) = &filter {
			if !name.starts_with(filter_str) {
				continue;
			}
		}

		println!("{name}");
	}

	Ok(())
}

fn cmd_init(name: String) -> Result<(), UtilityError> {
	if !BATL_NAME_REGEX.is_match(&name) {
		return Err(UtilityError::InvalidName(name));
	}

	let name: Name = name.into();

	Workspace::create(name.clone())?;

	success(&format!("Workspace {} initialized", name.clone()));

	Ok(())
}

fn cmd_delete(name: String) -> Result<(), UtilityError> {
	if !BATL_NAME_REGEX.is_match(&name) {
		return Err(UtilityError::InvalidName(name));
	}

	let workspace = Workspace::load(name.as_str().into())?
		.ok_or(UtilityError::ResourceDoesNotExist("Workspace".into()))?;

	workspace.destroy()?;

	success(&format!("Workspace {name} deleted"));

	Ok(())
}

fn cmd_which(name: String) -> Result<(), UtilityError> {
	if !BATL_NAME_REGEX.is_match(&name) {
		return Err(UtilityError::InvalidName(name));
	}

	let workspace = Workspace::load(name.into())?
		.ok_or(UtilityError::ResourceDoesNotExist("Workspace".into()))?;

	println!("{}", workspace.path().to_string_lossy());

	Ok(())
}
