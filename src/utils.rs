use batl::error as batlerror;
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

#[cfg(target_os = "windows")]
use crate::output::error;


lazy_static! {
	pub static ref BATL_NAME_REGEX: Regex = Regex::new(r"^[a-z][a-z0-9\-_]*(/[a-z][a-z0-9\-_]*)+$").unwrap();
	pub static ref BATL_LINK_REGEX: Regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_\-]*$").unwrap();
}

#[derive(Error, Debug)]
pub enum UtilityError {
	#[error("IO Error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Resource does not exist: {0}")]
	ResourceDoesNotExist(String),
	#[error("Resource already exists: {0}")]
	ResourceAlreadyExists(String),
	#[error("Invalid config")]
	InvalidConfig,
	#[error("Link not found")]
	LinkNotFound,
	#[error("Invalid name: {0}")]
	InvalidName(String),
	#[error("Already setup")]
	AlreadySetup,
	#[error("Script not found: {0}")]
	ScriptNotFound(String),
	#[error("Script error: {0}")]
	ScriptError(String),
	#[error("Resource cannot be collected: {0}")]
	ResourceNotCollected(String),
	#[error("Network Error: {0}")]
	NetworkError(#[from] ureq::Error),
	#[error("Unknown")]
	Unknown
}

impl From<batlerror::ReadConfigError> for UtilityError {
	fn from(value: batlerror::ReadConfigError) -> Self {
		match value {
			batlerror::ReadConfigError::IoError(e) => e.into(),
			batlerror::ReadConfigError::TomlError(_) => UtilityError::InvalidConfig,
			_ => UtilityError::Unknown
		}
	}
}

impl From<batlerror::GeneralResourceError> for UtilityError {
	fn from(value: batlerror::GeneralResourceError) -> Self {
		match value {
			batlerror::GeneralResourceError::DoesNotExist => UtilityError::ResourceDoesNotExist("<>".to_string()),
			batlerror::GeneralResourceError::Invalid => UtilityError::InvalidConfig,
			batlerror::GeneralResourceError::IoError(e) => e.into(),
			_ => UtilityError::Unknown
		}
	}
}

impl From<batlerror::CreateResourceError> for UtilityError {
	fn from(value: batlerror::CreateResourceError) -> Self {
		match value {
			batlerror::CreateResourceError::AlreadyExists => UtilityError::ResourceAlreadyExists("<>".to_string()),
			batlerror::CreateResourceError::IoError(e) => e.into(),
			batlerror::CreateResourceError::NotSetup => UtilityError::ResourceAlreadyExists("Battalion root".to_string()),
			_ => UtilityError::Unknown
		}
	}
}

impl From<batlerror::DeleteResourceError> for UtilityError {
	fn from(value: batlerror::DeleteResourceError) -> Self {
		match value {
			batlerror::DeleteResourceError::DoesNotExist => UtilityError::ResourceAlreadyExists("<>".to_string()),
			batlerror::DeleteResourceError::IoError(e) => e.into(),
			_ => UtilityError::Unknown
		}
	}
}

impl From<batlerror::CreateDependentResourceError> for UtilityError {
	fn from(value: batlerror::CreateDependentResourceError) -> Self {
		match value {
			batlerror::CreateDependentResourceError::Creation(e) => e.into(),
			batlerror::CreateDependentResourceError::IoError(e) => e.into(),
			batlerror::CreateDependentResourceError::Dependent(e) => e.into(),
			_ => UtilityError::Unknown
		}
	}
}

#[cfg(target_os = "windows")]
pub fn windows_symlink_perms() -> Result<(), std::io::Error> {
	let winuser = whoami::username();
	let powershell_args = format!(
		r#"secedit /export /cfg c:\\secpol.cfg; (gc C:\\secpol.cfg).replace('SeCreateSymbolicLinkPrivilege = ', 'SeCreateSymbolicLinkPrivilege = "{}",') | Out-File C:\\secpol.cfg; secedit /configure /db c:\\windows\\security\\local.sdb /cfg c:\\secpol.cfg; rm -force c:\\secpol.cfg -confirm:$false"#,
		winuser
	);

	let powershell = std::process::Command::new("powershell.exe")
		.arg(powershell_args)
		.status()?;

	if !powershell.success() {
		error("Could not get symlink perms");
		std::process::exit(1);
	}

	Ok(())
}
