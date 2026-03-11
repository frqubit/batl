use batl_macros::ToFromLuaValue;
use semver::Version;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use crate::resource::Name;

#[derive(ToFromLuaValue)]
pub struct DownloadActionBatlConstant {
    pub handler: BatlConstantHandler,
    pub target: BatlConstantTarget,
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
pub struct BatlConstantTargetConfig {
    #[lua_serde]
    pub name: Option<Name>,
    #[lua_serde]
    pub version: Option<Version>,
}

#[derive(ToFromLuaValue)]
pub struct TargetExecuteOutput {
    status: i32,
    stdout: String,
}

pub fn target_execute(lua: &mlua::Lua, exec_dir: &Path) -> mlua::Result<mlua::Function> {
    let exec_dir = exec_dir.to_path_buf();

    lua.create_function(move |_, cmd: String| {
        if let Ok(args) = shellish_parse::parse(&cmd, false) {
            if args.len() == 0 {
                return Ok(TargetExecuteOutput {
                    status: 255,
                    stdout: Default::default(),
                });
            }

            let mut args = args.into_iter();

            let command = Command::new(args.next().unwrap())
                .args(args)
                .current_dir(exec_dir.clone())
                .output()?;

            Ok(TargetExecuteOutput {
                status: command.status.code().unwrap_or(255),
                stdout: String::from_utf8(command.stdout).unwrap_or_default(),
            })
        } else {
            Ok(TargetExecuteOutput {
                status: 255,
                stdout: Default::default(),
            })
        }
    })
}
