use crate::error::*;
use crate::error::{err_resource_does_not_exist, err_script_execution_failed};
use crate::output::{error, info, success};
use crate::resource::batlrc::AnyBatlRc;
use crate::resource::tomlconfig::{write_toml, TomlConfig};
use crate::resource::{self, Name, Repository};
use crate::resource::{batlrc::BatlRcLatest, BatlRc};
use crate::utils::REGISTRY_DOMAIN;
use colored::*;
use console::Term;
use fs_extra::dir::CopyOptions;
use git2::build::RepoBuilder;
use git2::{FetchOptions, Progress, RemoteCallbacks};
use itertools::Itertools;
use semver::Version;
use std::env::current_dir;
use std::io::Write;
use std::path::PathBuf;

pub mod repository;

fn print_versions(name: Name) -> EyreResult<()> {
    if name.version().is_some() {
        return Err(err_input_requested_is_invalid(
            &name.to_string(),
            "cannot list versions of versioned repository",
        ));
    }

    let repo_root = crate::system::repository_root().ok_or(err_battalion_not_setup())?;
    let fetched_root = crate::system::fetched_repository_root().ok_or(err_battalion_not_setup())?;

    let without_version = name.clone().without_version();

    let local_base_path = repo_root.join(without_version.path_segments_as_repository_name());
    let local_versioned_path = repo_root.join(without_version.path_segments_as_version_folder());
    let fetched_path = fetched_root.join(without_version.path_segments_as_version_folder());

    let mut local_base_repo_exists = false;

    // First print the local base version, if it exists
    if let Ok(local_base_repo) = Repository::from_path(&local_base_path) {
        local_base_repo_exists = true;

        println!(
            "{} : {}",
            &without_version,
            local_base_repo.config().version
        );
    }

    // Next, get all of the versions local and fetched
    let local_entries = std::fs::read_dir(local_versioned_path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let fetched_entries = std::fs::read_dir(fetched_path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let versions = local_entries
        .iter()
        .map(|val| (true, val))
        .chain(fetched_entries.iter().map(|val| (false, val)))
        .filter_map(|(is_local, val)| {
            Version::parse(&val.replace("__", "+"))
                .ok()
                .map(|ver| (is_local, ver))
        })
        .sorted_by(|a, b| Ord::cmp(&a.1, &b.1))
        .collect::<Vec<_>>();

    if versions.is_empty() && !local_base_repo_exists {
        return Err(err_resource_does_not_exist(&without_version.to_string()));
    }

    for (is_local, version) in versions {
        let name = name.clone().with_version(version);
        if is_local {
            println!("{}", name.to_string());
        } else {
            println!("{}", name.to_string().bright_red());
        }
    }

    Ok(())
}

pub fn cmd_ls(filter: Option<String>, versions: bool) -> EyreResult<()> {
    let repo_root = crate::system::repository_root().ok_or(err_battalion_not_setup())?;

    if versions {
        if let Some(v) = filter {
            let name = Name::new(&v)?;
            return print_versions(name);
        } else {
            return Err(err_input_requested_is_invalid(
                "empty filter",
                "forced version requires a repository name",
            ));
        }
    }

    let filter_path = filter
        .clone()
        .map(|v| Name::new(&v).map(|v| Name::path_segments_as_folder_name(&v)))
        .transpose()?
        .transpose()?
        .unwrap_or_default();

    let search_path = repo_root.join(filter_path);

    if !search_path.exists() {
        if let Some(v) = filter {
            let name = Name::new(&v)?;
            return print_versions(name);
        } else {
            return Ok(());
        }
    }

    let names = std::fs::read_dir(search_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    for name in names.iter() {
        if let Some(versioned) = name.strip_prefix("__") {
            if names.contains(&versioned.into()) {
                println!("{}", versioned.green().italic());
            } else {
                println!("{}", versioned.green());
            }
        } else if let Some(folder) = name.strip_prefix("_") {
            println!("{}", folder.blue());
        } else if !names.contains(&format!("__{name}")) {
            println!("{}", name.italic())
        }
    }

    // let found = std::fs::read_dir(search_path)?
    //     .filter_map(|entry| entry.ok())
    //     .map(|entry| {
    //         let name_os = entry.file_name();
    //         let name = name_os.to_string_lossy();

    //         if let Some(folder) = name.strip_prefix('_') {
    //             folder.blue()
    //         } else {
    //             name.italic()
    //         }
    //     });

    // for name in found {
    //     println!("{name}");
    // }

    //

    // let mut to_search: Vec<(String, PathBuf)> = std::fs::read_dir(repo_root)?
    // 	.filter_map(|entry| {
    // 		Some(("".to_string(), entry.ok()?.path()))
    // 	})
    // 	.collect();
    // let mut found: Vec<String> = Vec::new();

    // while let Some((name, path)) = to_search.pop() {
    // 	if !path.is_dir() {
    // 		continue;
    // 	}

    // 	let filename = path.file_name().unwrap().to_str().unwrap();

    // 	if let Some(filename) = filename.strip_prefix('_') {
    // 		let new_name = filename.to_string();
    // 		let new_name = format!("{name}{new_name}.");

    // 		to_search.extend(
    // 			std::fs::read_dir(path)?
    // 				.filter_map(|entry| {
    // 					Some((new_name.clone(), entry.ok()?.path()))
    // 				})
    // 		);
    // 	} else {
    // 		found.push(format!("{name}{filename}"));
    // 	}
    // }

    Ok(())
}

pub fn cmd_init(name: String) -> EyreResult<()> {
    let name = Name::new(&name)?;

    Repository::create(name, Default::default())?;

    success("Initialized repository successfully");

    Ok(())
}

pub fn cmd_delete(name: String) -> EyreResult<()> {
    let confirmation = dialoguer::Confirm::new()
        .with_prompt("This deletion is permanent, are you sure you want to continue?")
        .interact()
        .unwrap();

    if confirmation {
        let repository =
            Repository::load(Name::new(&name)?)?.ok_or(err_resource_does_not_exist(&name))?;

        let mut path = repository.path().to_path_buf();

        repository.destroy()?;

        while let Some(parent) = path.parent() {
            if std::fs::read_dir(parent)?.count() == 0 {
                std::fs::remove_dir(parent)?;
                path = parent.to_path_buf();
            } else {
                break;
            }
        }

        success("Deleted repository successfully");
    } else {
        info("cancelled");
    }

    Ok(())
}

pub fn cmd_search(name: Option<String>) -> EyreResult<()> {
    let name = name.map(|v| Name::new(&v)).transpose()?;

    let name_query = name
        .map(|v| format!("?q={}", v.url_path_segments()))
        .unwrap_or("".into());
    let url = format!("{REGISTRY_DOMAIN}/pkg{name_query}");

    let body = ureq::get(&url).call()?.body_mut().read_to_string()?;

    let items: Vec<String> = serde_json::from_str(&body)?;

    for item in items {
        if let Some(repo) = item.strip_suffix(".tar") {
            println!("{}", repo.italic());
        } else {
            println!("{}", item.blue());
        }
    }

    Ok(())
}

pub fn cmd_publish(name: String) -> EyreResult<()> {
    let batlrc: BatlRc = crate::system::batlrc()?
        .ok_or(err_battalion_not_setup())?
        .into();

    let repository =
        Repository::load(Name::new(&name)?)?.ok_or(err_resource_does_not_exist(&name))?;

    // let archive = repository.archive_gen()?;

    let archive = repository
        .archive()?
        .ok_or(err_resource_does_not_exist("archive"))?;

    let url = format!(
        "{REGISTRY_DOMAIN}/pkg/{}",
        &repository.name().url_path_segments()
    );

    let resp = ureq::post(&url)
        .header("x-api-key", &batlrc.api.credentials)
        .send(archive.to_file())?;

    if resp.status() == 200 {
        success(&format!("Published repository {name}"))
    } else {
        error(&format!(
            "Failed to send repository: status code {}",
            resp.status()
        ))
    }

    Ok(())
}

#[derive(Default)]
pub struct CmdFetchOptions {
    pub git: bool,
    pub local: bool
}

fn transfer_progress(progress: Progress<'_>) -> bool {
    let percentage = progress.received_objects() as f64 / progress.total_objects() as f64;

    let mut term = Term::stdout();

    term.clear_line().unwrap();
    term.write_fmt(format_args!(
        "Cloning repository... {:.2}%",
        percentage * 100.0
    ))
    .unwrap();
    term.flush().unwrap();

    true
}

pub fn cmd_fetch(name: String, options: CmdFetchOptions) -> EyreResult<()> {
    let dummy_folder = match options.git {
        false => {
            let name = Name::new(&name)?;

            let url = format!("{REGISTRY_DOMAIN}/pkg/{}", name.url_path_segments());
        
            let resp = ureq::get(&url).call()?;
        
            if resp.status() != 200 {
                error(&format!(
                    "Failed to fetch repository: status code {}",
                    resp.status()
                ));
                return Ok(());
            }
        
            let body = resp.into_body().into_reader();
        
            let mut tar = tar::Archive::new(body);
        
            // The repository needs to be unpacked
            // to a dummy folder first, then moved
            // to its final destination
            let dummy_folder = tempfile::tempdir()?;
            tar.unpack(&dummy_folder)?;

            dummy_folder
        },
        true => {
            let dummy_folder = tempfile::tempdir()?;

            let mut fetch_callbacks = RemoteCallbacks::new();
            fetch_callbacks.transfer_progress(transfer_progress);
    
            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks(fetch_callbacks);
    
            RepoBuilder::new()
                .fetch_options(fetch_options)
                .clone(&name, dummy_folder.path())?;
    
            success("Successfully fetched repository");

            dummy_folder
        }
    };

    // Get the version of the package
    let repo_config =
        resource::repository::Config::from(resource::repository::TomlConfigLatest::from(
            resource::repository::AnyTomlConfig::read_toml(&dummy_folder.path().join("batl.toml"))?,
        ));
    let version = repo_config.version;
    let name = repo_config.name.with_version(version);

    let repository_path = match options.local {
        true => crate::system::repository_root(),
        false => crate::system::fetched_repository_root()
    }.ok_or(err_battalion_not_setup())?.join(name.path_segments_as_repository_name());

    std::fs::create_dir_all(&repository_path)?;

    fs_extra::dir::move_dir(
        dummy_folder,
        &repository_path,
        &CopyOptions::new().content_only(true),
    )?;

    // std::fs::rename(dummy_folder.keep(), &repository_path)?;

    success(&format!("Fetched repository {name}"));

    Ok(())
}

pub fn cmd_exec(name: Option<String>, script: String, args: Vec<String>) -> EyreResult<()> {
    let repository = match &name {
        Some(val) => Repository::load(Name::new(val)?)?,
        None => Repository::locate_then_load(&current_dir()?)?,
    };

    if let Some(repository) = repository {
        let command = repository
            .script(&script)
            .ok_or(err_script_does_not_exist(&script))?;

        info(&format!(
            "Running script{}",
            name.map(|s| format!(" for link {s}"))
                .unwrap_or("".to_string())
        ));

        let batl_pwd = current_dir()?;

        let status = std::process::Command::new("sh")
            .current_dir(repository.path())
            .env("BATL_PWD", batl_pwd.as_os_str())
            .arg("-c")
            .arg(command)
            .arg("batl-executor")
            .args(args)
            .status()?;

        if !status.success() {
            return Err(err_script_execution_failed(
                &script,
                status.code().unwrap_or(0),
            ));
        }

        success("Script completed successfully");

        Ok(())
    } else {
        Err(match &name {
            Some(v) => err_resource_does_not_exist(v),
            None => err_not_executed_inside_repository(),
        })
    }
}

pub fn cmd_which(name: Option<String>) -> EyreResult<()> {
    if let Some(name) = name {
        let repository =
            Repository::load(Name::new(&name)?)?.ok_or(err_resource_does_not_exist(&name))?;

        println!("{}", repository.path().to_string_lossy());
    } else {
        let repository = Repository::locate_then_load(&current_dir()?)?
            .ok_or(err_not_executed_inside_repository())?;

        println!("{}", repository.name());
    }

    Ok(())
}

pub fn cmd_setup() -> EyreResult<()> {
    #[cfg(target_os = "windows")]
    crate::utils::windows_symlink_perms()?;

    if crate::system::batl_root().is_some() {
        // If installed already then just update instead
        cmd_upgrade()?;
        return Ok(());
    }

    let batl_root = dirs::home_dir()
        .ok_or(err_missing_system_ability("system user directory"))?
        .join("battalion");

    std::fs::create_dir_all(batl_root.join("repositories"))?;

    let batlrc = BatlRc::default();

    write_toml(&batl_root.join(".batlrc"), &batlrc)?;

    println!(
        "Battalion root directory created at {}",
        batl_root.display()
    );

    Ok(())
}

pub fn cmd_add(name: String) -> EyreResult<()> {
    let name = Name::new(&name)?;

    let mut repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;

    repository.add_dependency(&name, None)?;

    success(&format!("Added dependency {name}"));

    Ok(())
}

pub fn cmd_remove(name: String) -> EyreResult<()> {
    let name = Name::new(&name)?;

    let mut repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;

    repository.remove_dependency(&name)?;

    success(&format!("Removed dependency {name}"));

    Ok(())
}

fn migrate_at_to_underscore(path: &PathBuf) -> EyreResult<()> {
    let mut to_search: Vec<PathBuf> = std::fs::read_dir(path)?
        .filter_map(|entry| Some(entry.ok()?.path()))
        .collect();

    while let Some(path) = to_search.pop() {
        if !path.is_dir() {
            continue;
        }

        let filename = path.file_name().unwrap().to_str().unwrap();

        if filename.strip_prefix('@').is_some() {
            migrate_at_to_underscore(&path)?;
        }
    }

    let this_filename = path.file_name().unwrap().to_string_lossy();

    if let Some(noprefix) = this_filename.strip_prefix('@') {
        let path_parent = path.parent().unwrap().to_path_buf();
        let new_path = path_parent.join(format!("_{noprefix}"));

        std::fs::rename(path, new_path)?;
    }

    Ok(())
}

pub fn cmd_upgrade() -> EyreResult<()> {
    let batl_root = crate::system::batl_root().ok_or(err_battalion_not_setup())?;

    if !batl_root.join("gen").exists() {
        let gen_ = batl_root.join("gen");

        std::fs::create_dir(&gen_)?;
        std::fs::create_dir(gen_.join("archives"))?;
        std::fs::create_dir(gen_.join("archives/repositories"))?;
        std::fs::create_dir(gen_.join("fetched"))?;

        success("Added gen folder");
    }

    if crate::system::batlrc()?.is_none() {
        // migrate @ to _
        migrate_at_to_underscore(&crate::system::repository_root().ok_or(
            err_internal_structure_malformed("missing repository root yet battalion root exists"),
        )?)?;

        let batlrc = BatlRc::default();

        write_toml(
            &crate::system::batlrc_path().expect("Nonsensical already checked for root"),
            &batlrc,
        )?;

        success("Added batlrc toml");
    }

    if let Some(AnyBatlRc::V0_2_1(v021)) = crate::system::batlrc()? {
        // migrate @ to _
        migrate_at_to_underscore(&crate::system::repository_root().ok_or(
            err_internal_structure_malformed("missing repository root yet battalion root exists"),
        )?)?;

        write_toml(
            &crate::system::batlrc_path().expect("Nonsensical already checked for root"),
            &BatlRcLatest::from(v021),
        )?;

        success("migrated batlrc to 0.3.0");
    }

    Ok(())
}

pub fn cmd_auth() -> EyreResult<()> {
    let key_prompt = dialoguer::Input::new();

    let api_key: String = key_prompt.with_prompt("API key").interact_text()?;

    let mut batlrc: BatlRc = crate::system::batlrc()?
        .ok_or(err_battalion_not_setup())?
        .into();

    batlrc.api.credentials = api_key;

    write_toml(
        &crate::system::batlrc_path().expect("Nonsensical just read batlrc"),
        &batlrc,
    )?;

    success("Added new API key");

    Ok(())
}

pub fn cmd_link(name: String, path: PathBuf) -> EyreResult<()> {
    let mut repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;
    let other = Repository::load(Name::new(&name)?)?.ok_or(err_resource_does_not_exist(&name))?;

    repository.add_link(&other, path)?;

    success("Added new link");

    Ok(())
}

pub fn cmd_unlink(name: String) -> EyreResult<()> {
    let mut repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;

    repository.remove_link(&Name::new(&name)?)?;

    success(&format!("Removed link for {name}"));
    Ok(())
}

pub fn cmd_deps() -> EyreResult<()> {
    let repository = Repository::locate_then_load(&current_dir()?)?
        .ok_or(err_not_executed_inside_repository())?;

    let summary = repository.summarize()?;

    for dep in summary.dependencies.into_iter() {
        let name = dep.0.with_version(dep.1);
        println!("{name}");
    }

    Ok(())
}
