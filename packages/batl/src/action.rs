use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use mlua::prelude::*;
use semver::Version;

use crate::error::{
    err_action_script_failed, err_resource_does_not_exist, err_resource_does_not_have_thing,
};
use crate::resource::Repository;
use crate::EyreResult;

pub fn run_action(
    action_repo: &Repository,
    target_repo: &Repository,
    action: String,
) -> EyreResult<()> {
    let action_path =
        action_repo
            .config()
            .actions
            .get(&action)
            .ok_or(err_resource_does_not_have_thing(
                &format!("repository {}", action_repo.name().to_string()),
                &format!("action {action}"),
            ))?;

    // Add repository path to action path
    let action_path = action_repo.path().join(action_path);

    if !action_path.exists() {
        return Err(err_resource_does_not_exist(
            &action_path.to_string_lossy().to_string(),
        ));
    }

    let mut action_file = File::open(action_path)?;
    let mut action_data: String = String::new();

    action_file.read_to_string(&mut action_data)?;

    // Start lua environment
    let lua = Lua::new();

    lua.load(action_data)
        .exec()
        .map_err(|e| err_action_script_failed(&e.to_string()))?;

    Ok(())
}
