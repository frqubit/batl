use std::{env::current_dir, path::PathBuf, str::FromStr};

use clap::{Args, Parser, Subcommand};
use color_eyre::{eyre::eyre, Result as EyreResult};
use error::err_not_executed_inside_repository;
use resource::Repository;

use crate::resource::{Name, SubpathableName};

mod action;
mod commands;
mod error;
mod output;
mod resource;
mod system;
mod utils;
mod version;

#[derive(Parser)]
#[command(name = "batl")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "The multi-repo development tool")]
struct Cli {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    #[command(about = "[DEPRECATED 0.3.0] Old repository command aliases")]
    Repository(SubCmdArgs<commands::repository::Commands>),
    #[cfg(debug_assertions)]
    #[command(about = "Development commands")]
    Dev(SubCmdArgs<commands::dev::Commands>),
    #[command(about = "Lists present battalion repositories")]
    Ls {
        filter: Option<Name>,
        #[arg(short = 'v')]
        versions: bool,
    },
    #[command(about = "Creates a new battalion repository")]
    Init { name: Name },
    #[command(about = "Deletes a battalion repository (be careful!)")]
    Delete { name: Name },
    #[command(about = "Publishes a repository")]
    Publish { name: String },
    #[command(about = "Gets the system path to a repository or name of current repository")]
    Which {
        name: Option<Name>,
        #[arg(short = 'v')]
        version: bool,
    },
    #[command(about = "Executes a command on a repository")]
    Exec {
        #[arg(short = 'n')]
        name: Option<Name>,
        script: String,
        args: Vec<String>,
    },
    #[command(about = "Sets up battalion")]
    Setup,
    #[command(about = "Adds a dependency")]
    Add { name: Name },
    #[command(about = "Removes a dependency")]
    #[command(alias = "rm")]
    Remove { name: Name },
    #[command(about = "Upgrades the installed battalion to the newest version")]
    Upgrade,
    #[command(about = "Adds an API key")]
    Auth,
    #[command(about = "Search registry for repositories")]
    Search { name: Option<Name> },
    #[command(about = "Links a dependency to a folder")]
    Link {
        name: SubpathableName,
        path: PathBuf,
    },
    #[command(about = "Unlinks a depenency from a folder")]
    Unlink { name: SubpathableName },
    #[command(about = "Lists dependencies of the current repository")]
    Deps,
    #[command(about = "Generates batl.lock")]
    Lock,
    #[command(about = "Pulls a repository using a handler")]
    Pull { name: Name, config: Vec<String> },
    #[command(external_subcommand)]
    ExecShorthand(Vec<String>),
}

#[derive(Args)]
struct SubCmdArgs<T: Subcommand> {
    #[command(subcommand)]
    subcmd: T,
}

fn main() -> EyreResult<()> {
    let cli = Cli::parse();

    let result = match cli.subcmd {
        SubCommand::Repository(args) => commands::repository::run(args.subcmd),
        #[cfg(debug_assertions)]
        SubCommand::Dev(args) => commands::dev::run(args.subcmd),
        SubCommand::Setup => commands::cmd_setup(),
        SubCommand::Add { name } => commands::cmd_add(name),
        SubCommand::Remove { name } => commands::cmd_remove(name),
        SubCommand::Upgrade => commands::cmd_upgrade(),
        SubCommand::Auth => commands::cmd_auth(),
        SubCommand::Ls { filter, versions } => commands::cmd_ls(filter, versions),
        SubCommand::Init { name } => commands::cmd_init(name),
        SubCommand::Delete { name } => commands::cmd_delete(name),
        SubCommand::Publish { name } => commands::cmd_publish(name),
        SubCommand::Exec { name, script, args } => commands::cmd_exec(name, script, args),
        SubCommand::Which { name, version } => commands::cmd_which(name, version),
        SubCommand::Search { name } => commands::cmd_search(name),
        SubCommand::Link { name, path } => commands::cmd_link(name, path),
        SubCommand::Unlink { name } => commands::cmd_unlink(name),
        SubCommand::Deps => commands::cmd_deps(),
        SubCommand::Lock => commands::cmd_lock(),
        SubCommand::Pull { name, config } => commands::cmd_pull(name, config),
        SubCommand::ExecShorthand(args) => cmd_execshorthand(args),
    };

    if let Err(err) = result {
        output::error(err.to_string().as_str());
        if cfg!(debug_assertions) {
            return Err(err);
        }
    }

    Ok(())
}

fn cmd_execshorthand(args: Vec<String>) -> EyreResult<()> {
    let mut args = args.into_iter();
    let resource = args
        .next()
        .ok_or(eyre!("Shorthand exec requires resource argument"))?;

    if let Some((name, cmd)) = resource.split_once(':') {
        if name.is_empty() {
            let name = Repository::locate_then_load(&current_dir()?)?
                .ok_or(err_not_executed_inside_repository())?;

            commands::cmd_exec(Some(name.name().clone()), cmd.into(), args.collect())
        } else {
            commands::cmd_exec(Some(Name::from_str(name)?), cmd.into(), args.collect())
        }
    } else {
        commands::cmd_exec(
            Some(Name::from_str(&resource)?),
            "exec".into(),
            args.collect(),
        )
    }
}
