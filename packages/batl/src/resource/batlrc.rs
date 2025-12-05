use batl_macros::versioned_identical;
use serde::{Serialize, Deserialize};

// TODO make all of these parseable
versioned_identical!("0.3.0" => "latest" : [BatlRc]);
versioned_identical!("0.2.2" => "0.3.0" : [BatlRc]);
versioned_identical!("0.2.1" => "0.2.2" : [BatlRc]);

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[expect(clippy::exhaustive_structs)]
pub struct BatlRc0_2_1 {
	pub api: Api0_2_1
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[expect(clippy::exhaustive_structs)]
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
