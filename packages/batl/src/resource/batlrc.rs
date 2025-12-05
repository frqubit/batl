use serde::{Serialize, Deserialize};


pub type BatlRcLatest = BatlRc0_2_2;
pub type BatlRc0_2_2 = BatlRc0_2_1;


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
