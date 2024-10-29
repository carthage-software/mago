use serde_json::from_str;
use serde_json::Error;

pub use crate::schema::*;

pub mod schema;

impl ComposerPackage {
    pub fn from_str(json: &str) -> Result<ComposerPackage, Error> {
        from_str(json)
    }
}
