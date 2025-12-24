use super::{
    tomlconfig::{self, RestrictRequirement0_2_2},
    Name,
};
use std::collections::HashMap;

/// A condition that restricts usage of a repository
#[derive(Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Condition {
    Windows,
    Linux,
    Unix,
    MacOs,
}

impl From<Condition> for tomlconfig::RestrictorLatest {
    #[inline]
    fn from(value: Condition) -> Self {
        match value {
            Condition::Linux => Self::Linux,
            Condition::MacOs => Self::MacOs,
            Condition::Unix => Self::Unix,
            Condition::Windows => Self::Windows,
        }
    }
}

impl From<tomlconfig::Restrictor0_2_2> for Condition {
    #[inline]
    fn from(value: tomlconfig::Restrictor0_2_2) -> Self {
        match value {
            tomlconfig::Restrictor0_2_2::Linux => Self::Linux,
            tomlconfig::Restrictor0_2_2::Windows => Self::Windows,
            tomlconfig::Restrictor0_2_2::MacOs => Self::MacOs,
            tomlconfig::Restrictor0_2_2::Unix => Self::Unix,
        }
    }
}

/// Settings for a battalion condition
#[derive(Clone)]
#[non_exhaustive]
pub struct Settings {
    pub include: Requirement,
    pub dependencies: HashMap<Name, String>,
}

impl From<Settings> for tomlconfig::RestrictorSettings0_2_2 {
    #[inline]
    fn from(value: Settings) -> Self {
        let include = Some(RestrictRequirement0_2_2::from(value.include)).and_then(|req| {
            if req == tomlconfig::RestrictRequirement0_2_2::Allow {
                None
            } else {
                Some(req)
            }
        });

        Self {
            include,
            dependencies: tomlconfig::hashmap_to_option_hashmap(value.dependencies),
        }
    }
}

impl From<tomlconfig::RestrictorSettings0_2_2> for Settings {
    #[inline]
    fn from(value: tomlconfig::RestrictorSettings0_2_2) -> Self {
        let include = value.include.map_or(Requirement::Allow, Requirement::from);

        Self {
            include,
            dependencies: value.dependencies.unwrap_or_default(),
        }
    }
}

/// Requirement severity of a condition
#[derive(Clone)]
#[non_exhaustive]
pub enum Requirement {
    Deny,
    Allow,
    Require,
}

impl From<Requirement> for tomlconfig::RestrictRequirement0_2_2 {
    #[inline]
    fn from(value: Requirement) -> Self {
        match value {
            Requirement::Allow => Self::Allow,
            Requirement::Deny => Self::Deny,
            Requirement::Require => Self::Require,
        }
    }
}

impl From<tomlconfig::RestrictRequirement0_2_2> for Requirement {
    #[inline]
    fn from(value: tomlconfig::RestrictRequirement0_2_2) -> Self {
        match value {
            tomlconfig::RestrictRequirement0_2_2::Allow => Self::Allow,
            tomlconfig::RestrictRequirement0_2_2::Deny => Self::Deny,
            tomlconfig::RestrictRequirement0_2_2::Require => Self::Require,
        }
    }
}
