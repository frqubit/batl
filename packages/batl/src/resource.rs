use crate::error::{err_battalion_not_setup, err_input_requested_is_invalid, EyreResult};
use core::fmt::{Display, Formatter};
use core::str::FromStr;
use semver::Version;
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{self, Serialize};
use std::path::{Path, PathBuf};

pub mod archive;
pub mod batlrc;
pub mod repository;
pub mod restrict;
// pub mod summary;
pub mod tomlconfig;

pub use self::batlrc::BatlRcLatest as BatlRc;
pub use self::repository::Repository;

/// A Battalion resource name
///
/// These are used for repositories, workspaces, and
/// their archives
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    segments: Vec<String>,
    version: Option<Version>,
}

impl Name {
    /// Get the path components of a name
    #[must_use]
    pub const fn segments(&self) -> &Vec<String> {
        &self.segments
    }

    pub const fn version(&self) -> &Option<Version> {
        &self.version
    }

    pub fn with_version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }

    pub fn without_version(mut self) -> Self {
        self.version = None;
        self
    }

    pub fn new(val: &str) -> EyreResult<Self> {
        let mut next = String::new();
        let mut segments = vec![];
        let mut version = String::new();
        let mut doing_version = false;

        for c in val.chars() {
            if doing_version {
                version.push(c);
                continue;
            }

            if c == '@' {
                // Last segment will be pushed later, DO NOT do it now
                doing_version = true;
                continue;
            }

            if next.is_empty() && c == '_' {
                return Err(err_input_requested_is_invalid(
                    val,
                    "Name segment cannot start with an underscore",
                ));
            }

            if c == '.' {
                if next.is_empty() {
                    return Err(err_input_requested_is_invalid(
                        val,
                        "Name segment cannot end with a period",
                    ));
                }

                // Start a new segment
                segments.push(next);
                next = String::new();
                continue;
            }

            // Add any valid characters to the segment
            if c.is_alphanumeric() || c == '-' || c == '_' {
                next.push(c);
            } else {
                return Err(err_input_requested_is_invalid(
                    val,
                    "Segment parts must be alphanumeric",
                ));
            }
        }

        // If segment is empty then either last c was
        // a period or the input was empty
        if next.is_empty() {
            return Err(err_input_requested_is_invalid(
                val,
                "Name cannot end with a period or be empty",
            ));
        }

        segments.push(next);

        let version = match version.is_empty() {
            true => None,
            false => Some(Version::parse(&version)?),
        };

        Ok(Self { segments, version })
    }

    pub fn from_absolute_path(value: &Path) -> EyreResult<Self> {
        if !value.is_absolute() {
            return Err(err_input_requested_is_invalid(
                &value.to_string_lossy(),
                "Path to convert to a name must be absolute",
            ));
        }

        let repository_root = crate::system::repository_root().ok_or(err_battalion_not_setup())?;
        let fetched_repository_root =
            crate::system::fetched_repository_root().ok_or(err_battalion_not_setup())?;
        let mut subpath = value.strip_prefix(repository_root);
        if subpath.is_err() {
            subpath = value.strip_prefix(fetched_repository_root)
        }

        if let Ok(subpath) = subpath {
            let mut segments = vec![];
            let mut version = None;
            let mut doing_version = false;

            let components = subpath
                .components()
                .map(|v| v.as_os_str().to_string_lossy());

            for component in components {
                if doing_version && version.is_none() {
                    version = Some(Version::parse(&component.replace("__", "+"))?);
                    continue;
                }

                if doing_version && version.is_some() {
                    return Err(err_input_requested_is_invalid(
                        &value.to_string_lossy(),
                        "version must be last component of path",
                    ));
                }

                if let Some(val) = component.strip_prefix("__") {
                    doing_version = true;
                    segments.push(val.to_string());
                } else if let Some(val) = component.strip_prefix("_") {
                    segments.push(val.to_string());
                }
            }

            let segments = subpath
                .components()
                .map(|v| v.as_os_str().to_string_lossy())
                .map(|v| v.strip_prefix("_").unwrap_or(&v).to_string())
                .collect();

            return Ok(Name { segments, version });
        }

        Err(err_input_requested_is_invalid(
            &value.to_string_lossy(),
            "Path to convert to a name must be a child of the repository root",
        ))
    }

    #[must_use]
    pub fn path_segments_as_folder_name(&self) -> EyreResult<PathBuf> {
        if self.version.is_some() {
            return Err(err_input_requested_is_invalid(
                &format!("{self}"),
                "names with versions cannot be used as folders",
            ));
        }

        Ok(self.segments().iter().map(|v| format!("_{v}")).collect())
    }

    #[must_use]
    pub fn path_segments_as_version_folder(&self) -> PathBuf {
        let parts = self.segments();

        let mut path = PathBuf::new();

        let mut parts_rev = parts.iter().rev();
        let last = parts_rev.next();

        let parts_without_end = parts_rev.rev();

        for part in parts_without_end {
            path = path.join(format!("_{part}"));
        }

        // last is guaranteed to be Some() because of checks on Name::new
        // TODO: This is safe but make this a type-guaranteed check, maybe add a `last` field to name?
        path = path.join(last.map(|v| format!("__{v}")).unwrap_or_default());

        path
    }

    #[must_use]
    pub fn path_segments_as_repository_name(&self) -> PathBuf {
        let parts = self.segments();

        let mut path = PathBuf::new();

        let mut parts_rev = parts.iter().rev();
        let last = parts_rev.next();

        let parts_without_end = parts_rev.rev();

        for part in parts_without_end {
            path = path.join(format!("_{part}"));
        }

        if let Some(version) = &self.version {
            // last is guaranteed to be Some() because of checks on Name::new
            // TODO: This is safe but make this a type-guaranteed check, maybe add a `last` field to name?
            path = path
                .join(format!("__{}", last.cloned().unwrap_or_default()))
                .join(version.to_string().replace('+', "__"));
        } else {
            path = path.join(last.cloned().unwrap_or_default());
        }

        path
    }

    #[must_use]
    pub fn url_path_segments(&self) -> String {
        let segments = self.segments.join("/");
        let version = self
            .version
            .as_ref()
            .map(|version| format!("/_v{}", version.to_string().replace("+", "__")))
            .unwrap_or_default();

        format!("{segments}{version}")
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
        f.write_str(&self.segments.join("."))?;

        if let Some(version) = &self.version {
            f.write_fmt(format_args!("@{version}"))?;
        }

        Ok(())
    }
}

#[expect(clippy::missing_trait_methods, reason = "serde autoimpls methods")]
impl<'de> Deserialize<'de> for Name {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
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
                E: de::Error,
            {
                Name::from_str(v).map_err(|_| {
                    de::Error::invalid_value(de::Unexpected::Str(v), &"A valid resource name")
                })
            }
        }

        deserializer.deserialize_str(NameVisitor)
    }
}

impl Serialize for Name {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
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
