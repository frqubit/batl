use std::env::current_dir;

use crate::{
    error::err_not_executed_inside_repository, output::success, resource::Repository, EyreResult,
};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    HashId,
}

pub fn run(cmd: Commands) -> EyreResult<()> {
    match cmd {
        Commands::HashId => cmd_hashid(),
    }
}

fn cmd_hashid() -> EyreResult<()> {
    let repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;

    repository.gen_hashid()?;

    Ok(())
}
