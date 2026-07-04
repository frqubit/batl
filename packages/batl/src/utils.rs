use crate::error::EyreResult;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};
use xxhash_rust::xxh3::Xxh3;

#[cfg(target_os = "windows")]
use crate::output::error;

pub const REGISTRY_DOMAIN: &str = "https://api.batl.circetools.net";

// #[derive(Error, Debug)]
// pub enum UtilityError {
// 	#[error("IO Error: {0}")]
// 	IoError(#[from] std::io::Error),
// 	#[error("Resource does not exist: {0}")]
// 	ResourceDoesNotExist(String),
// 	#[error("Resource already exists: {0}")]
// 	ResourceAlreadyExists(String),
// 	#[error("Invalid config")]
// 	InvalidConfig,
// 	#[error("Invalid JSON from API")]
// 	InvalidApiJson(#[from] serde_json::Error),
// 	#[error("Link not found")]
// 	LinkNotFound,
// 	#[error("Invalid name: {0}")]
// 	InvalidName(String),
// 	#[error("Already setup")]
// 	AlreadySetup,
// 	#[error("Script not found: {0}")]
// 	ScriptNotFound(String),
// 	#[error("Script error: {0}")]
// 	ScriptError(String),
// 	#[error("Resource cannot be collected: {0}")]
// 	ResourceNotCollected(String),
// 	#[error("Network Error: {0}")]
// 	NetworkError(#[from] Box<ureq::Error>),
// 	#[error("Terminal input error: {0}")]
// 	TerminalInputError(#[from] dialoguer::Error),
// 	#[error("{0}")]
// 	InvalidValue(#[from] InvalidValueError),
// 	#[error("Unknown")]
// 	Unknown
// }

// impl From<batlerror::ReadConfigError> for UtilityError {
// 	fn from(value: batlerror::ReadConfigError) -> Self {
// 		match value {
// 			batlerror::ReadConfigError::IoError(e) => e.into(),
// 			batlerror::ReadConfigError::TomlError(_) => UtilityError::InvalidConfig,
// 			_ => UtilityError::Unknown
// 		}
// 	}
// }

// impl From<batlerror::GeneralResourceError> for UtilityError {
// 	fn from(value: batlerror::GeneralResourceError) -> Self {
// 		match value {
// 			batlerror::GeneralResourceError::DoesNotExist => UtilityError::ResourceDoesNotExist("<>".to_string()),
// 			batlerror::GeneralResourceError::Invalid => UtilityError::InvalidConfig,
// 			batlerror::GeneralResourceError::IoError(e) => e.into(),
// 			_ => UtilityError::Unknown
// 		}
// 	}
// }

// impl From<batlerror::CreateResourceError> for UtilityError {
// 	fn from(value: batlerror::CreateResourceError) -> Self {
// 		match value {
// 			batlerror::CreateResourceError::AlreadyExists => UtilityError::ResourceAlreadyExists("<>".to_string()),
// 			batlerror::CreateResourceError::IoError(e) => e.into(),
// 			batlerror::CreateResourceError::NotSetup => UtilityError::ResourceAlreadyExists("Battalion root".to_string()),
// 			_ => UtilityError::Unknown
// 		}
// 	}
// }

// impl From<batlerror::DeleteResourceError> for UtilityError {
// 	fn from(value: batlerror::DeleteResourceError) -> Self {
// 		match value {
// 			batlerror::DeleteResourceError::DoesNotExist => UtilityError::ResourceAlreadyExists("<>".to_string()),
// 			batlerror::DeleteResourceError::IoError(e) => e.into(),
// 			_ => UtilityError::Unknown
// 		}
// 	}
// }

// impl From<batlerror::CreateDependentResourceError> for UtilityError {
// 	fn from(value: batlerror::CreateDependentResourceError) -> Self {
// 		match value {
// 			batlerror::CreateDependentResourceError::Creation(e) => e.into(),
// 			batlerror::CreateDependentResourceError::IoError(e) => e.into(),
// 			batlerror::CreateDependentResourceError::Dependent(e) => e.into(),
// 			_ => UtilityError::Unknown
// 		}
// 	}
// }

// impl From<ureq::Error> for UtilityError {
// 	fn from(value: ureq::Error) -> Self {
// 		UtilityError::NetworkError(Box::new(value))
// 	}
// }

pub fn genhash_of_directory_as_repository(path: &Path) -> EyreResult<String> {
    let mut walk_builder = ignore::WalkBuilder::new(path);

    walk_builder.add_ignore(".batl/*");
    walk_builder.add_custom_ignore_filename("batl.ignore");

    let walk = walk_builder.build();

    let mut hasher = Sha256::new();
    let mut files: Vec<(String, String)> = vec![];

    for result in walk {
        let entry = result?;

        let abs_path = entry.path();

        if abs_path.is_dir() {
            continue;
        }

        let rel_path_opt = pathdiff::diff_paths(abs_path, path);

        if let Some(rel_path) = rel_path_opt {
            let filename = rel_path.to_string_lossy().to_string();
            let mut file_hasher = Xxh3::new();
            let mut buffer = [0; 1024];

            let mut file = File::open(abs_path)?;
            while let Ok(n) = file.read(&mut buffer) {
                if n == 0 {
                    // EOF
                    break;
                }

                file_hasher.update(&buffer[..n]);
            }

            let file_hash = format!("{:X}", file_hasher.digest128());

            let pos = files.iter().position(|file| file.0 > filename);
            if let Some(pos) = pos {
                files.insert(pos, (filename, file_hash));
            } else {
                files.push((filename, file_hash));
            }
        }
    }

    for (filename, hash) in files.iter() {
        hasher.update(format!("{filename} {hash}").as_bytes());
    }

    let hash = format!("{:X}", hasher.finalize());

    Ok(hash)
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
