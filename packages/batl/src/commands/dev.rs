use std::{collections::HashMap, env::current_dir};

use crate::{
    error::{
        err_check_failed, err_input_requested_is_invalid, err_not_executed_inside_repository,
        err_resource_does_not_exist, err_theoretical,
    },
    resource::{
        source::{CheckPullResult, RepositorySource},
        Name, Repository,
    },
    EyreResult,
};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    HashId,
    Sources,
    CheckPull,
}

pub fn run(cmd: Commands) -> EyreResult<()> {
    match cmd {
        Commands::HashId => cmd_hashid(),
        Commands::Sources => cmd_sources(),
        Commands::CheckPull => cmd_check_pull(),
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

fn cmd_check_pull() -> EyreResult<()> {
    let repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;

    let repo_summary = repository.summarize()?;
    let deps = repo_summary.dependencies;

    for dep in deps.into_iter() {
        let sources = dep.clone().sources.into_iter().map(RepositorySource::from);
        let mut found_good_source = false;

        for source in sources {
            let check_res = source.check_pull(&dep)?;

            if let CheckPullResult::CanPullAndDid(res) = check_res {
                let name = res.0.name.unwrap();
                let path = res.1.cwd.path();
                let hashid = crate::utils::genhash_of_directory_as_repository(path)?;

                crate::output::info(&format!("HASHID {}: {}", name, hashid));

                if
                /* dep.name.clone().without_version() == name.without_version() && */
                hashid == dep.hashid {
                    found_good_source = true;
                    break;
                }
            } else {
                if let CheckPullResult::CanPullButDidNot = check_res {
                    found_good_source = true;
                    break;
                }
            }
        }

        if !found_good_source {
            return Err(err_check_failed(&format!(
                "{} does not have a satisfactory source",
                &dep.name
            )));
        }
    }

    crate::output::success("Repository is confirmed pullable");

    Ok(())
}
