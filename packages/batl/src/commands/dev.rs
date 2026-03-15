use rand::Rng;
use std::{collections::HashMap, env::current_dir};

use crate::{
    action::DownloadAction,
    error::{
        err_input_requested_is_invalid, err_not_executed_inside_repository,
        err_resource_does_not_exist, err_theoretical,
    },
    resource::{source::RepositorySource, Name, Repository},
    EyreResult,
};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    HashId,
    Sources,
    Download { name: Name, config: Vec<String> },
}

pub fn run(cmd: Commands) -> EyreResult<()> {
    match cmd {
        Commands::HashId => cmd_hashid(),
        Commands::Sources => cmd_sources(),
        Commands::Download { name, config } => cmd_download(name, config),
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

pub fn cmd_download(name: Name, config: Vec<String>) -> EyreResult<()> {
    let repository =
        Repository::load(name.clone())?.ok_or(err_resource_does_not_exist(&name.to_string()))?;

    let mut attrs = HashMap::new();

    for config_val in config {
        if let Some((name, val)) = config_val.split_once('=') {
            attrs.insert(name.to_string(), val.to_string());
        } else {
            return Err(err_input_requested_is_invalid(
                &config_val,
                "config values must have '='",
            ));
        }
    }

    let source = RepositorySource {
        handler: name,
        attrs,
    };

    let batl_gen_path = crate::system::gen_root().unwrap();
    let rand_code = rand::rng()
        .random_iter::<u32>()
        .take(8)
        .map(|v| char::from_u32('A' as u32 + v % 26).unwrap())
        .collect::<String>();

    let tempdir_path = batl_gen_path.join("temp").join(rand_code);
    let tempdir = std::fs::create_dir_all(&tempdir_path);

    let action = DownloadAction {
        source: source.clone(),
        download_to: tempdir_path.clone(),
    };

    let data = crate::action::run_action(&repository, action);

    if let Ok(data) = data {
        let repo_name = data.name.unwrap();
        let repo_version = data.version.unwrap();

        let mut new_repo =
            Repository::create(repo_name.clone().with_version(repo_version.clone()))?;

        for entry in std::fs::read_dir(&tempdir_path)? {
            let entry = entry?;
            let source_path = entry.path();
            let file_name = source_path.file_name().ok_or_else(err_theoretical)?;
            let destination_path = new_repo.path().join(file_name);

            std::fs::rename(&source_path, &destination_path)?;
        }
        std::fs::remove_dir_all(tempdir_path)?;

        new_repo.reload()?;

        // [TODO] The name could be changed, account for this now but in the future this needs to be forbidden
        new_repo.config_mut().name = repo_name.clone();
        new_repo.config_mut().version = repo_version.clone();
        new_repo.config_mut().sources.push(source);
        new_repo.save()?;
    } else {
        std::fs::remove_dir_all(tempdir_path)?;
        data?;
    }

    Ok(())
}
