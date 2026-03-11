use mlua::prelude::*;
use std::fs::File;
use std::io::Read;

use crate::error::{
    err_action_script_failed, err_resource_does_not_exist, err_resource_does_not_have_thing,
};
use crate::resource::source::RepositorySource;
use crate::resource::{repository, Repository};
use crate::EyreResult;

pub mod batlconstant;

trait BatlAction {
    type BatlActionBatlConstant: IntoLua + FromLua;

    fn batl_action_batl_constant(&self, lua: &Lua) -> EyreResult<Self::BatlActionBatlConstant>;
    fn function_name(&self) -> &'static str;
}

pub struct DownloadAction<'life> {
    pub source: RepositorySource,
    pub target_repo: Option<&'life repository::Repository>,
}

impl<'life> BatlAction for DownloadAction<'life> {
    type BatlActionBatlConstant = batlconstant::DownloadActionBatlConstant;

    fn batl_action_batl_constant(&self, lua: &Lua) -> EyreResult<Self::BatlActionBatlConstant> {
        let config = self.target_repo.unwrap().config();

        Ok(batlconstant::DownloadActionBatlConstant {
            handler: batlconstant::BatlConstantHandler {
                data: self.source.attrs.clone(),
            },
            target: batlconstant::BatlConstantTarget {
                execute: batlconstant::target_execute(
                    lua,
                    self.target_repo.map(|f| f.path()).unwrap(),
                )?,
                config: batlconstant::BatlConstantTargetConfig {
                    name: Some(config.name.clone()),
                    version: Some(config.version.clone()),
                },
            },
        })
    }

    fn function_name(&self) -> &'static str {
        "download"
    }
}

pub fn run_action<A>(action_repo: &Repository, action: A) -> EyreResult<()>
where
    A: BatlAction,
{
    let action_path =
        action_repo
            .config()
            .actions_filepath
            .clone()
            .ok_or(err_resource_does_not_have_thing(
                &format!("repository {}", action_repo.name().to_string()),
                "action file",
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

    let lua_function: mlua::Function = lua.globals().get(action.function_name())?;
    lua_function.call::<()>(action.batl_action_batl_constant(&lua)?)?;

    Ok(())
}
