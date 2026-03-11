use std::{collections::HashMap, env::current_dir};

use crate::{
    action::DownloadAction,
    error::err_not_executed_inside_repository,
    resource::{source::RepositorySource, Name, Repository},
    EyreResult,
};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    HashId,
    Sources,
    DownloadAct,
}

pub fn run(cmd: Commands) -> EyreResult<()> {
    match cmd {
        Commands::HashId => cmd_hashid(),
        Commands::Sources => cmd_sources(),
        Commands::DownloadAct => cmd_download_act(),
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

pub fn cmd_download_act() -> EyreResult<()> {
    let repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;

    let mut attrs = HashMap::new();
    attrs.insert(
        "url".to_string(),
        "https://github.com/frqubit/batl".to_string(),
    );

    let source = RepositorySource {
        handler: Name::new("battalion.source.github")?,
        attrs,
    };

    let action = DownloadAction {
        source,
        target_repo: Some(&repository),
    };

    crate::action::run_action(&repository, action)?;

    Ok(())
}
