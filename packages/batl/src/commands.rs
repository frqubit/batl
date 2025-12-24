use crate::error::*;
use crate::error::{err_resource_does_not_exist, err_script_execution_failed};
use crate::output::{error, info, success};
use crate::resource::batlrc::AnyBatlRc;
use crate::resource::tomlconfig::write_toml;
use crate::resource::{batlrc::BatlRcLatest, BatlRc};
use crate::resource::{Name, Repository};
use crate::utils::REGISTRY_DOMAIN;
use colored::*;
use std::env::current_dir;
use std::path::PathBuf;

pub mod repository;

pub fn cmd_ls(filter: Option<String>) -> EyreResult<()> {
    let repo_root = crate::system::repository_root().ok_or(err_battalion_not_setup())?;

    let filter_path = filter
        .map(|v| Name::new(&v).map(|v| Name::path_segments_as_folder_name(&v)))
        .transpose()?
        .unwrap_or_default();

    let search_path = repo_root.join(filter_path);

    if !search_path.exists() {
        return Ok(());
    }

    let found = std::fs::read_dir(search_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let name_os = entry.file_name();
            let name = name_os.to_string_lossy();

            if let Some(folder) = name.strip_prefix('_') {
                folder.blue()
            } else {
                name.italic()
            }
        });

    for name in found {
        println!("{name}");
    }

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
    Repository::load(Name::new(&name)?)?
        .ok_or(err_resource_does_not_exist(&name))?
        .destroy()?;

    success("Deleted repository successfully");

    Ok(())
}

pub fn cmd_search(name: Option<String>) -> EyreResult<()> {
    let name = name.map(|v| Name::new(&v)).transpose()?;

    let name_query = name.map(|v| format!("?q={v}")).unwrap_or("".into());
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

    let archive = repository.archive()?;

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

pub fn cmd_fetch(name: String) -> EyreResult<()> {
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

    let repository_path = crate::system::repository_root()
        .ok_or(err_battalion_not_setup())?
        .join(name.path_segments_as_repository_name());

    std::fs::create_dir_all(&repository_path)?;

    tar.unpack(repository_path)?;

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

pub fn cmd_which(name: String) -> EyreResult<()> {
    let repository =
        Repository::load(Name::new(&name)?)?.ok_or(err_resource_does_not_exist(&name))?;

    println!("{}", repository.path().to_string_lossy());

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
