use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use color_eyre::{eyre::eyre, Result as EyreResult};

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
    #[command(about = "Lists present battalion repositories")]
    Ls { filter: Option<String> },
    #[command(about = "Creates a new battalion repository")]
    Init { name: String },
    #[command(about = "Deletes a battalion repository (be careful!)")]
    Delete { name: String },
    #[command(about = "Publishes a repository")]
    Publish { name: String },
    #[command(about = "Fetches a repository from the battalion registry")]
    Fetch { name: String },
    #[command(about = "Gets the system path to a repository")]
    Which { name: String },
    #[command(about = "Executes a command on a repository")]
    Exec {
        #[arg(short = 'n')]
        name: Option<String>,
        script: String,
        args: Vec<String>,
    },
    #[command(about = "Sets up battalion")]
    Setup,
    #[command(about = "Adds a dependency")]
    Add { name: String },
    #[command(about = "Removes a dependency")]
    #[command(alias = "rm")]
    Remove { name: String },
    #[command(about = "Upgrades the installed battalion to the newest version")]
    Upgrade,
    #[command(about = "Adds an API key")]
    Auth,
    #[command(about = "Search registry for repositories")]
    Search { name: Option<String> },
    #[command(about = "Links a dependency to a folder")]
    Link { name: String, path: PathBuf },
    #[command(about = "Unlinks a depenency from a folder")]
    Unlink { name: String },
    #[command(about = "Lists dependencies of the current repository")]
    Deps,
    #[command(external_subcommand)]
    ExecShorthand(Vec<String>),
}

#[derive(Args)]
struct SubCmdArgs<T: Subcommand> {
    #[command(subcommand)]
    subcmd: T,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.subcmd {
        SubCommand::Repository(args) => commands::repository::run(args.subcmd),
        SubCommand::Setup => commands::cmd_setup(),
        SubCommand::Add { name } => commands::cmd_add(name),
        SubCommand::Remove { name } => commands::cmd_remove(name),
        SubCommand::Upgrade => commands::cmd_upgrade(),
        SubCommand::Auth => commands::cmd_auth(),
        SubCommand::Ls { filter } => commands::cmd_ls(filter),
        SubCommand::Init { name } => commands::cmd_init(name),
        SubCommand::Delete { name } => commands::cmd_delete(name),
        SubCommand::Publish { name } => commands::cmd_publish(name),
        SubCommand::Fetch { name } => commands::cmd_fetch(name),
        SubCommand::Exec { name, script, args } => commands::cmd_exec(name, script, args),
        SubCommand::Which { name } => commands::cmd_which(name),
        SubCommand::Search { name } => commands::cmd_search(name),
        SubCommand::Link { name, path } => commands::cmd_link(name, path),
        SubCommand::Unlink { name } => commands::cmd_unlink(name),
        SubCommand::Deps => commands::cmd_deps(),
        SubCommand::ExecShorthand(args) => cmd_execshorthand(args),
    };

    if let Err(err) = result {
        output::error(err.to_string().as_str());
        std::process::exit(1);
    }
}

fn cmd_execshorthand(args: Vec<String>) -> EyreResult<()> {
    let mut args = args.into_iter();
    let resource = args
        .next()
        .ok_or(eyre!("Shorthand exec requires resource argument"))?;

    if let Some((name, cmd)) = resource.split_once(':') {
        commands::cmd_exec(Some(name.into()), cmd.into(), args.collect())
    } else {
        commands::cmd_exec(Some(resource.clone()), "exec".into(), args.collect())
    }
}
