#![allow(clippy::module_name_repetitions, reason = "version structs named version, fix in future")]

use batl_macros::semver_struct_impl;


pub type VersionLatest = Version0_2_2;

semver_struct_impl!("0.2.0");
semver_struct_impl!("0.2.1");
semver_struct_impl!("0.2.2");
