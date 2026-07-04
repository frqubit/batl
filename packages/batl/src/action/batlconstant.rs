use crate::resource::tomlconfig::TomlConfig;
use batl_macros::ToFromLuaValue;
use semver::Version;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use crate::resource::Name;

#[derive(ToFromLuaValue)]
pub struct PullActionBatlConstant {
    pub handler: BatlConstantHandler,
    pub target: BatlConstantTarget,
}

#[derive(ToFromLuaValue)]
pub struct CheckPullActionBatlConstant {
    pub handler: BatlConstantHandler,
    pub target: BatlCheckConstantTarget,
    pub confirm: bool,
}

#[derive(ToFromLuaValue)]
pub struct BatlConstantHandler {
    pub data: HashMap<String, String>,
}

#[derive(ToFromLuaValue)]
pub struct BatlConstantTarget {
    pub config: BatlConstantTargetConfig,
    pub execute: mlua::Function,
}

#[derive(ToFromLuaValue)]
pub struct BatlCheckConstantTarget {
    pub config: BatlCheckConstantTargetConfig,
}

#[derive(ToFromLuaValue)]
pub struct BatlConstantTargetConfig {
    #[lua_serde]
    pub name: Option<Name>,
    #[lua_serde]
    pub version: Option<Version>,
    pub reload: mlua::Function,
}

#[derive(ToFromLuaValue)]
pub struct BatlCheckConstantTargetConfig {
    #[lua_serde]
    pub name: Name,
    #[lua_serde]
    pub version: Version,
    pub hashid: String,
}

#[derive(ToFromLuaValue)]
pub struct TargetExecuteOutput {
    status: i32,
    stdout: String,
}

pub fn target_execute(lua: &mlua::Lua, exec_dir: &Path) -> mlua::Result<mlua::Function> {
    let exec_dir = exec_dir.to_path_buf();

    lua.create_function(move |_, (cmd, catch_stdout): (String, Option<bool>)| {
        let catch_stdout = catch_stdout.unwrap_or(false);

        if let Ok(args) = shellish_parse::parse(&cmd, false) {
            if args.len() == 0 {
                return Ok(TargetExecuteOutput {
                    status: 255,
                    stdout: Default::default(),
                });
            }

            let mut args = args.into_iter();

            if catch_stdout {
                let command = Command::new(args.next().unwrap())
                    .args(args)
                    .current_dir(exec_dir.clone())
                    .output()?;

                Ok(TargetExecuteOutput {
                    status: command.status.code().unwrap_or(255),
                    stdout: String::from_utf8(command.stdout).unwrap_or_default(),
                })
            } else {
                let command = Command::new(args.next().unwrap())
                    .args(args)
                    .current_dir(exec_dir.clone())
                    .status()?;

                Ok(TargetExecuteOutput {
                    status: command.code().unwrap_or(255),
                    stdout: "".to_string(),
                })
            }
        } else {
            Ok(TargetExecuteOutput {
                status: 255,
                stdout: Default::default(),
            })
        }
    })
}

pub fn target_config_reload(lua: &mlua::Lua, exec_dir: &Path) -> mlua::Result<mlua::Function> {
    let exec_dir = exec_dir.to_path_buf();

    lua.create_function(move |_, config: mlua::Table| {
        let toml = crate::resource::repository::AnyTomlConfig::load(&exec_dir);

        if let Some(toml) = toml {
            let latest: crate::resource::repository::TomlConfigLatest = toml.into();
            config.set("name", latest.repository.name.to_string())?;
            config.set("version", latest.repository.version.to_string())?;

            Ok(true)
        } else {
            Ok(false)
        }
    })
}
