#![allow(clippy::module_name_repetitions)]

use color_eyre::eyre::{eyre, Report};
pub use color_eyre::Result as EyreResult;

pub fn err_battalion_not_setup() -> Report {
    eyre!("Battalion has not been set up, run `batl setup` to fix")
}

pub fn err_not_executed_inside_repository() -> Report {
    eyre!("This command must be executed while the current shell is in a repository")
}

pub fn err_resource_does_not_exist(resource: &str) -> Report {
    eyre!("{resource} does not exist yet operation was attempted")
}

pub fn err_resource_already_exists() -> Report {
    eyre!("This resource already exists!")
}

pub fn err_script_does_not_exist(script: &str) -> Report {
    eyre!("A script named {script} does not exist in the repository")
}

pub fn err_script_execution_failed(script: &str, exit_code: i32) -> Report {
    eyre!("Script {script} failed with exit code {exit_code}")
}

pub fn err_input_requested_is_invalid(specifics: &str, reason: &str) -> Report {
    eyre!("The input of {specifics} is invalid: {reason}")
}

pub fn err_missing_system_ability(ability: &str) -> Report {
    eyre!("Battalion cannot find/utilize a(n) {ability} from the system or current user")
}

pub fn err_internal_structure_malformed(specifics: &str) -> Report {
    eyre!("The internal battalion folder structure is malformed: {specifics}")
}

pub fn err_resource_does_not_have_thing(resource: &str, thing: &str) -> Report {
    eyre!("The resource {resource} doesn't have {thing}")
}

pub fn err_action_impossible_while_condition(ing_action: &str, condition: &str) -> Report {
    eyre!("{ing_action} cannot be performed while {condition}")
}

// #[derive(Debug, Error)]
// #[non_exhaustive]
// pub enum ReadConfigError {
// 	#[error("{0}")]
// 	IoError(#[from] std::io::Error),
// 	#[error("{0}")]
// 	TomlError(#[from] toml::de::Error)
// }

// #[derive(Debug, Error)]
// #[non_exhaustive]
// pub enum CreateResourceError {
// 	#[error("IO Error: {0}")]
// 	IoError(#[from] std::io::Error),
// 	#[error("Battalion not set up")]
// 	NotSetup,
// 	#[error("Resource already exists")]
// 	AlreadyExists
// }

// #[derive(Debug, Error)]
// #[non_exhaustive]
// pub enum CreateDependentResourceError {
// 	#[error("IO Error: {0}")]
// 	IoError(#[from] std::io::Error),
// 	#[error("Error while creating resource: {0}")]
// 	Creation(#[from] CreateResourceError),
// 	#[error("Error while getting dependents: {0}")]
// 	Dependent(#[from] GeneralResourceError)
// }

// #[derive(Debug, Error)]
// #[non_exhaustive]
// pub enum GeneralResourceError {
// 	#[error("IO Error: {0}")]
// 	IoError(#[from] std::io::Error),
// 	#[error("Resource does not exist")]
// 	DoesNotExist,
// 	#[error("Resource invalid/corrupted")]
// 	Invalid
// }

// impl From<ReadConfigError> for GeneralResourceError {
// 	#[inline]
// 	fn from(value: ReadConfigError) -> Self {
// 		match value {
// 			ReadConfigError::IoError(e) if {
// 				e.kind() == std::io::ErrorKind::NotFound
// 			} => Self::DoesNotExist,
// 			ReadConfigError::IoError(e) => e.into(),
// 			ReadConfigError::TomlError(_) => Self::Invalid
// 		}
// 	}
// }

// #[derive(Debug, Error)]
// #[non_exhaustive]
// pub enum DeleteResourceError {
// 	#[error("IO Error: {0}")]
// 	IoError(#[from] std::io::Error),
// 	#[error("Resource does not exist")]
// 	DoesNotExist
// }

// #[derive(Debug)]
// pub struct InvalidValueError(String);

// impl InvalidValueError {
// 	pub fn new(val: &str) -> Self {
// 		Self(val.into())
// 	}
// }

// impl std::error::Error for InvalidValueError {}
// impl std::fmt::Display for InvalidValueError {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		f.write_fmt(format_args!("Invalid value: {}", self.0))
// 	}
// }
