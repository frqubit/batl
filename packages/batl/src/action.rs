use mlua::prelude::*;
use semver::Version;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::error::{
    err_action_script_failed, err_resource_does_not_exist, err_resource_does_not_have_thing,
};
use crate::resource::source::RepositorySource;
use crate::resource::{repository, Name, Repository};
use crate::EyreResult;

pub mod batlconstant;

pub trait BatlAction {
    type BatlActionBatlConstant: IntoLua + FromLua;
    type BatlActionOutput;

    fn batl_action_batl_constant(&self, lua: &Lua) -> EyreResult<Self::BatlActionBatlConstant>;
    fn function_name(&self) -> &'static str;
    fn as_output(
        &self,
        batl_constant: Self::BatlActionBatlConstant,
    ) -> EyreResult<Self::BatlActionOutput>;
}

pub struct DownloadAction {
    pub source: RepositorySource,
    // pub target_repo: Option<&'life repository::Repository>,
    pub download_to: PathBuf,
}

impl BatlAction for DownloadAction {
    type BatlActionBatlConstant = batlconstant::DownloadActionBatlConstant;
    type BatlActionOutput = batlconstant::BatlConstantTargetConfig;

    fn batl_action_batl_constant(&self, lua: &Lua) -> EyreResult<Self::BatlActionBatlConstant> {
        Ok(batlconstant::DownloadActionBatlConstant {
            handler: batlconstant::BatlConstantHandler {
                data: self.source.attrs.clone(),
            },
            target: batlconstant::BatlConstantTarget {
                execute: batlconstant::target_execute(lua, &self.download_to)?,
                config: batlconstant::BatlConstantTargetConfig {
                    name: None,
                    version: None,
                },
            },
        })
    }

    fn function_name(&self) -> &'static str {
        "download"
    }

    fn as_output(
        &self,
        batl_constant: Self::BatlActionBatlConstant,
    ) -> EyreResult<Self::BatlActionOutput> {
        let config = batl_constant.target.config;

        if config.name.is_none() || config.version.is_none() {
            return Err(err_action_script_failed("name or version is none"));
        }

        Ok(config)
    }
}

pub fn run_action<A>(action_repo: &Repository, action: A) -> EyreResult<A::BatlActionOutput>
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
    lua.globals()
        .set("__batl_global", action.batl_action_batl_constant(&lua)?)?;
    let function_data: mlua::Value = lua.globals().get("__batl_global")?;

    lua_function.call::<()>(function_data)?;

    let function_data: A::BatlActionBatlConstant = lua.globals().get("__batl_global")?;

    action.as_output(function_data)
}
