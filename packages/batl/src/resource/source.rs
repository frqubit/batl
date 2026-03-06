use crate::resource::Name;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct RepositorySource {
    pub handler: Name,
    pub attrs: HashMap<String, String>,
}
