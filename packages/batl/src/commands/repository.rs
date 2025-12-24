use batl::resource::{Repository, repository};
use batl::resource::repository::CreateRepositoryOptions;
use batl::resource::tomlconfig::{TomlConfig, RepositoryGit0_2_2};
use clap::Subcommand;
use console::Term;
use crate::output::*;
use crate::utils::{UtilityError, BATL_NAME_REGEX};
use envfile::EnvFile;
use git2::{FetchOptions, RemoteCallbacks, Progress};
use git2::build::RepoBuilder;
use std::env::current_dir;
use std::io::Write;


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
	Clone {
		url: String,
		#[arg(short = 'o')]
		name: String
	},
	Scaffold,
	Env {
		#[arg(short = 'n')]
		name: Option<String>,
		var: String
	},
	Archive {
		name: String
	},
	Publish {
		name: String
	},
	Fetch {
		name: String
	},
	Which {
		name: String
	},
	Exec {
		#[arg(short = 'n')]
		name: Option<String>,
		script: String
	}
}

pub fn run(cmd: Commands) -> Result<(), UtilityError> {
	match cmd {
		Commands::Ls { filter } => {
			super::cmd_ls(filter)
		},
		Commands::Init { name } => {
			super::cmd_init(name)
		},
		Commands::Delete { name } => {
			super::cmd_delete(name)
		},
		Commands::Clone { url, name } => {
			cmd_clone(url, name)
		},
		Commands::Scaffold => {
			cmd_scaffold()
		},
		Commands::Env { name, var } => {
			cmd_env(name, var)
		},
		Commands::Archive { name } => {
			cmd_archive(name)
		},
		Commands::Publish { name } => {
			super::cmd_publish(name)
		},
		Commands::Fetch { name } => {
			super::cmd_fetch(name)
		},
		Commands::Which { name } => {
			super::cmd_which(name)
		},
		Commands::Exec { name, script } => {
			super::cmd_exec(name, script, vec![])
		}
	}
}

fn cmd_clone(url: String, name: String) -> Result<(), UtilityError> {
	if !BATL_NAME_REGEX.is_match(&name) {
		return Err(UtilityError::InvalidName(name));
	}

	Repository::create(
		name.into(),
		CreateRepositoryOptions::git(RepositoryGit0_2_2 {
			url,
			path: "git".to_string()
		})
	)?;

	success("Initialized repository clone successfully");

	Ok(())
}

fn cmd_scaffold() -> Result<(), UtilityError> {
	let repository = Repository::locate_then_load(&current_dir()?)?
		.ok_or(UtilityError::ResourceDoesNotExist("Repository".to_string()))?;

	let config = repository.config();

	if let Some(git) = config.git.clone() {
		let git_path = repository.path().join(git.path);

		let mut fetch_callbacks = RemoteCallbacks::new();
		fetch_callbacks.transfer_progress(transfer_progress);

		let mut fetch_options = FetchOptions::new();
		fetch_options.remote_callbacks(fetch_callbacks);

		let result = RepoBuilder::new()
			.fetch_options(fetch_options)
			.clone(&git.url, &git_path);

		println!();

		if let Err(err) = result {
			println!("{err}");

			return Err(UtilityError::ResourceNotCollected("Git remote".to_string()));
		}

		success("Successfully scaffolded repository");
	}

	Ok(())
}

fn transfer_progress(progress: Progress<'_>) -> bool {
	let percentage = progress.received_objects() as f64 / progress.total_objects() as f64;

	let mut term = Term::stdout();

	term.clear_line().unwrap();
	term.write_fmt(format_args!("Cloning repository... {:.2}%", percentage * 100.0)).unwrap();
	term.flush().unwrap();



	true
}

fn cmd_env(name: Option<String>, var: String) -> Result<(), UtilityError> {
	let mut workspace_dir = repository::TomlConfigLatest::locate(&current_dir()?)
		.ok_or(UtilityError::ResourceDoesNotExist("Workspace Configuration".to_string()))?;

	if let Some(name) = &name {
		workspace_dir.push(name);
	}

	let env_file = EnvFile::new(workspace_dir.join("batl.env"))
		.map_err(|_| UtilityError::ResourceDoesNotExist("Environment variables".to_string()))?;

	if let Some(val) = env_file.get(&var) {
		println!("{val}");
	}

	Ok(())
}

fn cmd_archive(name: String) -> Result<(), UtilityError> {
	let repository = Repository::load(name.as_str().into())?
		.ok_or(UtilityError::ResourceDoesNotExist("Repository".into()))?;

	repository.archive_gen()?;

	Ok(())
}
