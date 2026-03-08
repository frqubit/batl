use std::env::current_dir;

use crate::{error::err_not_executed_inside_repository, resource::Repository, EyreResult};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    HashId,
    Sources,
}

pub fn run(cmd: Commands) -> EyreResult<()> {
    match cmd {
        Commands::HashId => cmd_hashid(),
        Commands::Sources => cmd_sources(),
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
