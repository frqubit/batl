use batl::resource::{Repository, Resource, Workspace};
use clap::{Subcommand, ValueEnum};
use crate::utils::{UtilityError, BATL_LINK_REGEX, BATL_NAME_REGEX};
use crate::output::*;
use std::env::current_dir;

#[derive(Subcommand)]
pub enum Commands {
	Ls,
	Stats {
		#[arg(long = "get")]
		get: Option<StatsGet>,
		name: String
	},
	Init {
		#[arg(short = 'n', long = "name")]
		name: Option<String>,
		repo: String
	},
	Delete {
		name: String
	},
	Run {
		name: String,
		#[arg(last = true)]
		args: Vec<String>
	},
	Exec {
		#[arg(short = 'n')]
		name: Option<String>,
		script: String
	}
}

pub fn run(cmd: Commands) -> Result<(), UtilityError> {
	match cmd {
		Commands::Ls => {
			cmd_ls()
		},
		Commands::Stats { name, get } => {
			cmd_stats(name, get)
		},
		Commands::Init { name, repo } => {
			cmd_init(name, repo)
		},
		Commands::Delete { name } => {
			cmd_delete(name)
		},
		Commands::Run { name, args } => {
			cmd_run(name, args)
		},
		Commands::Exec { name, script } => {
			cmd_exec(name, script)
		}
	}
}

fn cmd_ls() -> Result<(), UtilityError> {
	let workspace = Workspace::locate_then_load(&current_dir()?)?
		.ok_or(UtilityError::ResourceDoesNotExist("Workspace".to_string()))?;

	let links = workspace.links();

	for link in links {
		println!("{}:\t\t{}", link.0, link.1);
	}

	Ok(())
}

#[derive(Clone, ValueEnum)]
#[clap(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatsGet {
	Name,
	Repository
}

fn cmd_stats(name: String, get: Option<StatsGet>) -> Result<(), UtilityError> {
	let workspace = Workspace::locate_then_load(&current_dir()?)?
		.ok_or(UtilityError::ResourceDoesNotExist("Workspace".to_string()))?;

	let repository = workspace.link(&name)
		.ok_or(UtilityError::LinkNotFound)?;
	
	let path = repository.path();

	match get {
		None => {
			println!("Link: {name}");
			println!("Repository: {}", path.display());
		},
		Some(StatsGet::Name) => println!("{name}"),
		Some(StatsGet::Repository) => println!("{}", path.display())
	}

	Ok(())
}

fn cmd_init(name: Option<String>, repo: String) -> Result<(), UtilityError> {
	if !BATL_NAME_REGEX.is_match(&repo) {
		return Err(UtilityError::InvalidName(repo));
	}

	// TODO: Make random string
	let name = name.unwrap_or_else(|| unimplemented!());

	if !BATL_LINK_REGEX.is_match(&name) {
		return Err(UtilityError::InvalidName(name));
	}

	let repo = Repository::load(repo.as_str().into())?
		.ok_or(UtilityError::ResourceDoesNotExist(format!("Repository {repo}")))?;

	let mut workspace = Workspace::locate_then_load(&current_dir()?)?
		.ok_or(UtilityError::ResourceDoesNotExist("Workspace".to_string()))?;

	workspace.create_link(&name, &repo)?;

	success(&format!("Initialized link {name}"));

	Ok(())
}

fn cmd_delete(name: String) -> Result<(), UtilityError> {
	let mut workspace = Workspace::locate_then_load(&current_dir()?)?
		.ok_or(UtilityError::ResourceDoesNotExist("Workspace".to_string()))?;

	workspace.unlink(&name)?;

	success(&format!("Deleted link {name}"));

	Ok(())
}

fn cmd_run(name: String, args: Vec<String>) -> Result<(), UtilityError> {
	let workspace = Workspace::locate_then_load(&current_dir()?)?
		.ok_or(UtilityError::ResourceDoesNotExist("Workspace".to_string()))?;

	let repository = workspace.link(&name)
		.ok_or(UtilityError::LinkNotFound)?;

	info(&format!("Running command for link {name}\n"));

	let status = std::process::Command::new(args.first().unwrap())
		.current_dir(repository.path())
		.args(args.iter().skip(1))
		.status()?;

	if !status.success() {
		return Err(UtilityError::ScriptError(format!("Exit code {}", status.code().unwrap_or(0))))
	}

	println!();
	success("Command completed successfully");

	Ok(())
}

fn cmd_exec(name: Option<String>, script: String) -> Result<(), UtilityError> {
	let repository = match &name {
		Some(val) => {
			let workspace = Workspace::locate_then_load(&current_dir()?)?
				.ok_or(UtilityError::ResourceDoesNotExist("Workspace".to_string()))?;

			workspace.link(val)
		},
		None => Repository::locate_then_load(&current_dir()?)?
	}.ok_or(UtilityError::ResourceDoesNotExist("Repository".to_string()))?;

	let command = repository.script(&script)
		.ok_or(UtilityError::ScriptNotFound(script))?;

	info(&format!("Running script{}\n", name.map(|s| format!(" for link {s}")).unwrap_or("".to_string())));

	let status = std::process::Command::new("sh")
		.current_dir(repository.path())
		.arg("-c")
		.arg(command)
		.status()?;


	if !status.success() {
		return Err(UtilityError::ScriptError(format!("Exit code {}", status.code().unwrap_or(0))))
	}

	println!();
	success("Script completed successfully");

	Ok(())
}
