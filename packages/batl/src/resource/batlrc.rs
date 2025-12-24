use batl_macros::versioned_identical;
use serde::{Serialize, Deserialize};
use crate::error::EyreResult;

use crate::{system::batlrc_path, version::Version0_3_0};

pub enum AnyBatlRc {
	V0_3_0(BatlRc0_3_0),
	// V0_2_2(BatlRc0_2_2), (is version equivalent)
	V0_2_1(BatlRc0_2_1)
}

impl AnyBatlRc {
	pub fn read_toml() -> EyreResult<Option<Self>> {
		if let Some(batlrc_path) = batlrc_path() {
			let config_str = std::fs::read_to_string(batlrc_path)?;

			if let Ok(v030) = toml::from_str(&config_str) {
				return Ok(Some(Self::V0_3_0(v030)));
			}

			return Ok(Some(Self::V0_2_1(toml::from_str(&config_str)?)));
		}

		Ok(None)
	}
}

impl From<AnyBatlRc> for BatlRcLatest {
	fn from(value: AnyBatlRc) -> Self {
		match value {
			AnyBatlRc::V0_2_1(v021) => v021.into(),
			AnyBatlRc::V0_3_0(v030) => v030
		}
	}
}

// TODO make all of these parseable
versioned_identical!("0.3.0" => "latest" : [BatlRc]);

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct BatlRc0_3_0 {
	pub version: Version0_3_0,
	pub api: Api0_2_1
}

versioned_identical!("0.2.1" => "0.2.2" : [BatlRc]);

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct BatlRc0_2_1 {
	pub api: Api0_2_1
}

impl From<BatlRc0_2_1> for BatlRcLatest {
	fn from(value: BatlRc0_2_1) -> Self {
		Self {
			version: Version0_3_0,
			api: value.api
		}
	}
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Api0_2_1 {
	pub credentials: String
}

impl Default for Api0_2_1 {
	#[inline]
	fn default() -> Self {
		Self {
			credentials: "YOUR-KEY-GOES-HERE".to_owned()
		}
	}
}
