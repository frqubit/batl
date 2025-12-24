use core::fmt::{Display, Formatter};
use core::str::FromStr;
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{self, Serialize};
use std::path::{Path, PathBuf};
use crate::error::{EyreResult, err_input_requested_is_invalid, err_battalion_not_setup};

pub mod archive;
pub mod batlrc;
pub mod repository;
pub mod restrict;
pub mod tomlconfig;

pub use self::archive::Archive;
pub use self::batlrc::BatlRcLatest as BatlRc;
pub use self::repository::Repository;

/// A Battalion resource name
/// 
/// These are used for repositories, workspaces, and
/// their archives
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(Vec<String>);

impl Name {
	/// Get the path components of a name
	#[must_use]
	pub const fn components(&self) -> &Vec<String> {
		&self.0
	}

	pub fn new(val: &str) -> EyreResult<Self> {
		let mut next = String::new();
		let mut segments = vec![];

		for c in val.chars() {
			if next.is_empty() && c == '_' {
				return Err(err_input_requested_is_invalid(val, "Name segment cannot start with an underscore"));
			}

			if c == '.' {
				if next.is_empty() {
					return Err(err_input_requested_is_invalid(val, "Name segment cannot end with a period"));
				}

				// Start a new segment
				segments.push(next);
				next = String::new();
				continue;
			}

			// Add any valid characters to the segment
			if c.is_alphanumeric() {
				next.push(c);
			}
		}

		// If segment is empty then either last c was
		// a period or the input was empty
		if next.is_empty() {
			return Err(err_input_requested_is_invalid(val, "Name cannot end with a period or be empty"));
		}

		segments.push(next);

		Ok(Self(segments))
	}

	pub fn from_absolute_path(value: &Path) -> EyreResult<Self> {
		if !value.is_absolute() {
			return Err(err_input_requested_is_invalid(&value.to_string_lossy(), "Path to convert to a name must be absolute"));
		}

		let repository_root = crate::system::repository_root()
			.ok_or(err_battalion_not_setup())?;

		if let Ok(subpath) = value.strip_prefix(repository_root) {
			let segments = subpath.components()
				.map(|v| v.as_os_str().to_string_lossy())
				.map(|v| v.strip_prefix("_").unwrap_or(&v).to_string())
				.collect();

			Self(segments);
		}

		Err(err_input_requested_is_invalid(&value.to_string_lossy(), "Path to convert to a name must be a child of the repository root"))
	}

	#[must_use] pub fn path_segments_as_folder_name(&self) -> PathBuf {
		self.components().iter().map(|v| format!("_{v}")).collect()
	}

	#[must_use] pub fn path_segments_as_repository_name(&self) -> PathBuf {
		let parts = self.components();

		let mut path = PathBuf::new();

		let mut parts_rev = parts.iter().rev();
		let last = parts_rev.next();

		let parts_without_end = parts_rev.rev();

		for part in parts_without_end {
			path = path.join(format!("_{part}"));
		}

		path = path.join(last.cloned().unwrap_or_default());

		path
	}

	#[must_use] pub fn url_path_segments(&self) -> String {
		self.0.join("/")
	}
}

impl FromStr for Name {
	type Err = color_eyre::eyre::Error;

	#[inline]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(s)
	}
}

impl Display for Name {
	#[inline]
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_str(&self.0.join("."))
	}
}

#[expect(clippy::missing_trait_methods, reason = "serde autoimpls methods")]
impl<'de> Deserialize<'de> for Name {
	#[inline]
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>
	{
		/// serde visitor for a battalion resource name
		struct NameVisitor;

		impl Visitor<'_> for NameVisitor {
			type Value = Name;

			fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
				formatter.write_str("A valid resource name")
			}

			#[expect(clippy::map_err_ignore, reason = "err specifics not important")]
			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: de::Error
			{
				Name::from_str(v)
					.map_err(|_| de::Error::invalid_value(de::Unexpected::Str(v), &"A valid resource name"))
			}
		}

		deserializer.deserialize_str(NameVisitor)
	}
}

impl Serialize for Name {
	#[inline]
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: ser::Serializer
	{
		serializer.serialize_str(&format!("{self}"))	
	}
}

/// Creates a symlink directory, OS independent
/// 
/// # Errors
/// 
/// Returns any IO errors that are received in the process
#[inline]
pub fn symlink_dir(original: &Path, link: &Path) -> Result<(), std::io::Error> {
	#[cfg(unix)]
	return std::os::unix::fs::symlink(original, link);

	#[cfg(target_os = "windows")]
	return std::os::windows::fs::symlink_dir(original, link);
}
