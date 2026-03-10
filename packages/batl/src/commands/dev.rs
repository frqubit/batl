use std::env::current_dir;

use crate::{error::err_not_executed_inside_repository, resource::Repository, EyreResult};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    HashId,
    Sources,
    Act { name: String },
}

pub fn run(cmd: Commands) -> EyreResult<()> {
    match cmd {
        Commands::HashId => cmd_hashid(),
        Commands::Sources => cmd_sources(),
        Commands::Act { name } => cmd_act(name),
    }
}

fn cmd_hashid() -> EyreResult<()> {
    let repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;

    let hash = repository.gen_hashid()?;
    println!("Hash: {hash}");

    Ok(())
}

fn cmd_sources() -> EyreResult<()> {
    let repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;

    let sources = repository.config().sources.clone();

    println!("{:?}", sources);

    Ok(())
}

pub fn cmd_act(name: String) -> EyreResult<()> {
    let repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;

    repository.run_action_on_repository(name, &repository)?;

    Ok(())
}
