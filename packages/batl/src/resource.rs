use core::convert::Infallible;
use core::fmt::{Display, Formatter};
use core::str::FromStr;
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{self, Serialize};
use std::path::{Path, PathBuf};

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

	/// Create a new battalion resource name
	#[must_use]
	pub const fn new(components: Vec<String>) -> Self {
		Self(components)
	}
}


impl From<&Name> for PathBuf {
	#[inline]
	fn from(value: &Name) -> Self {
		let parts = value.components();

		let mut path = Self::new();

		let mut parts_rev = parts.iter().rev();
		let last = parts_rev.next();

		let parts_without_end = parts_rev.rev();

		for part in parts_without_end {
			path = path.join(format!("_{part}"));
		}

		path = path.join(last.cloned().unwrap_or_default());

		path
	}
}

impl From<&Path> for Name {
	#[inline]
	fn from(path: &Path) -> Self {
		let mut value = path.iter();

		let mut parts = vec![value.next().unwrap_or_default().to_string_lossy().to_string()];

		for val in value {
			let val_string = val.to_string_lossy().to_string();

			if val_string.starts_with('_') {
				parts.push(val_string.get(1..).unwrap_or_default().to_owned());
			}
		}

		Self::new(parts)
	}
}

impl FromStr for Name {
	type Err = Infallible;

	#[inline]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::new(s.split('.').map(ToString::to_string).collect()))
	}
}

#[expect(clippy::fallible_impl_from, reason = "FIX fromstr is infallible")]
impl From<String> for Name {
	#[inline]
	fn from(value: String) -> Self {
		Self::from_str(&value).unwrap()
	}
}

#[expect(clippy::fallible_impl_from, reason = "FIX fromstr is infallible")]
impl From<&str> for Name {
	#[inline]
	fn from(value: &str) -> Self {
		Self::from_str(value).unwrap()
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
