use crate::error::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use super::Name;


pub struct Archive {
	/// The tar file
	pub(crate) tar: tar::Archive<File>,

	/// The path of the tar file
	pub(crate) path: PathBuf
}

impl Archive {
	/// Load the archive with the supplied name
	/// 
	/// # Errors
	/// 
	/// Returns any errors that come up while getting the resource.
	/// Also returns None if the resource does not exist
	#[inline]
	pub fn load(name: &Name) -> EyreResult<Option<Self>> {
		let slashed_name = name.to_string().replace('.', "/");

		let tar_path = crate::system::archive_root().map(|p| p
			.join("repositories")
			.join(format!("{slashed_name}.tar"))
		);

		if let Some(tar_path) = tar_path {
			let file = File::open(&tar_path)?;
			let archive = tar::Archive::new(file);


			Ok(Some(Self {
				path: tar_path,
				tar: archive
			}))
		} else {
			Ok(None)
		}
	}

	#[inline]
	pub const fn tar(&self) -> &tar::Archive<File> {
		&self.tar
	}

	#[inline]
	pub fn path(&self) -> &Path {
		&self.path
	}

	#[inline]
	pub fn to_file(self) -> File {
		self.tar.into_inner()
	}
}
