use mlua::prelude::*;
use semver::Version;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::error::{
    err_action_script_failed, err_resource_does_not_exist, err_resource_does_not_have_thing,
};
use crate::resource::source::RepositorySource;
use crate::resource::summary::SummarizedDependency;
use crate::resource::{Name, Repository};
use crate::EyreResult;

pub mod batlconstant;

pub trait BatlAction {
    type BatlActionBatlConstant: IntoLua + FromLua;
    type BatlActionOutput;

    fn batl_action_batl_constant(
        &self,
        lua: &Lua,
        env: &ActionEnv,
    ) -> EyreResult<Self::BatlActionBatlConstant>;
    fn function_name(&self) -> &'static str;
    fn as_output(
        &self,
        batl_constant: Self::BatlActionBatlConstant,
    ) -> EyreResult<Self::BatlActionOutput>;
}

pub struct ActionEnv {
    pub cwd: TempDir,
}

pub struct PullAction {
    pub source: RepositorySource,
}

impl BatlAction for PullAction {
    type BatlActionBatlConstant = batlconstant::PullActionBatlConstant;
    type BatlActionOutput = batlconstant::BatlConstantTargetConfig;

    fn batl_action_batl_constant(
        &self,
        lua: &Lua,
        env: &ActionEnv,
    ) -> EyreResult<Self::BatlActionBatlConstant> {
        Ok(batlconstant::PullActionBatlConstant {
            handler: batlconstant::BatlConstantHandler {
                data: self.source.attrs.clone(),
            },
            target: batlconstant::BatlConstantTarget {
                path: None,
                execute: batlconstant::target_execute(lua, &env.cwd.path())?,
                write: batlconstant::target_write(lua, &env.cwd.path())?,
                config: batlconstant::BatlConstantTargetConfig {
                    name: None,
                    version: None,
                    reload: batlconstant::target_config_reload(lua, &env.cwd.path())?,
                },
            },
        })
    }

    fn function_name(&self) -> &'static str {
        "pull"
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

pub struct CheckPullAction {
    pub source: RepositorySource,
    pub name: Name,
    pub version: Version,
    pub hashid: String,
}

impl BatlAction for CheckPullAction {
    type BatlActionBatlConstant = batlconstant::CheckPullActionBatlConstant;
    type BatlActionOutput = bool;

    fn function_name(&self) -> &'static str {
        "check_pull"
    }

    fn batl_action_batl_constant(
        &self,
        lua: &Lua,
        env: &ActionEnv,
    ) -> EyreResult<Self::BatlActionBatlConstant> {
        Ok(batlconstant::CheckPullActionBatlConstant {
            handler: batlconstant::BatlConstantHandler {
                data: self.source.attrs.clone(),
            },
            target: batlconstant::BatlCheckConstantTarget {
                path: None,
                config: batlconstant::BatlCheckConstantTargetConfig {
                    name: self.name.clone().without_version(),
                    version: self.version.clone(),
                    hashid: self.hashid.clone(),
                },
            },
            confirm: false,
        })
    }

    fn as_output(
        &self,
        batl_constant: Self::BatlActionBatlConstant,
    ) -> EyreResult<Self::BatlActionOutput> {
        Ok(batl_constant.confirm)
    }
}

pub struct PushAction<'life> {
    pub source: RepositorySource,
    pub repository: &'life Repository,
}

impl<'life> BatlAction for PushAction<'life> {
    type BatlActionBatlConstant = batlconstant::PushActionBatlConstant;
    type BatlActionOutput = ();

    fn function_name(&self) -> &'static str {
        "push"
    }

    fn batl_action_batl_constant(
        &self,
        lua: &Lua,
        env: &ActionEnv,
    ) -> EyreResult<Self::BatlActionBatlConstant> {
        Ok(batlconstant::PushActionBatlConstant {
            handler: batlconstant::BatlConstantHandler {
                data: self.source.attrs.clone(),
            },
            target: batlconstant::BatlConstantTarget {
                path: Some(self.repository.path().to_path_buf()),
                execute: batlconstant::target_execute(lua, &env.cwd.path())?,
                write: batlconstant::target_write(lua, &env.cwd.path())?,
                config: batlconstant::BatlConstantTargetConfig {
                    name: Some(self.repository.config().name.clone().without_version()),
                    version: Some(self.repository.config().version.clone()),
                    reload: batlconstant::target_config_reload(lua, &env.cwd.path())?,
                },
            },
            manual: batlconstant::BatlConstantManual {
                confirm: batlconstant::manual_confirm(lua)?,
            },
        })
    }

    fn as_output(
        &self,
        _batl_constant: Self::BatlActionBatlConstant,
    ) -> EyreResult<Self::BatlActionOutput> {
        Ok(())
    }
}

pub fn repository_can_execute_action<A>(action_repo: &Repository, action: &A) -> EyreResult<bool>
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

    let lua_function: mlua::Value = lua.globals().get(action.function_name())?;
    return Ok(!lua_function.is_nil());
}

pub fn run_action<A>(
    action_repo: &Repository,
    action: A,
) -> EyreResult<(A::BatlActionOutput, ActionEnv)>
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

    // Make temporary folder
    let batl_temp_path = crate::system::gen_root().unwrap().join("temp");
    let temp_dir = tempfile::TempDir::new_in(&batl_temp_path)?;

    // Get action env
    let action_env = ActionEnv { cwd: temp_dir };

    lua.load(action_data)
        .exec()
        .map_err(|e| err_action_script_failed(&e.to_string()))?;

    let lua_function: mlua::Function = lua.globals().get(action.function_name())?;
    lua.globals().set(
        "__batl_global",
        action.batl_action_batl_constant(&lua, &action_env)?,
    )?;
    let function_data: mlua::Value = lua.globals().get("__batl_global")?;

    lua_function.call::<()>(function_data)?;

    let function_data: A::BatlActionBatlConstant = lua.globals().get("__batl_global")?;

    action.as_output(function_data).map(|d| (d, action_env))
}
