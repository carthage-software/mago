use std::borrow::Cow;
use std::path::PathBuf;

use crate::error::DatabaseError;

#[derive(Debug)]
pub(crate) enum FilesystemOperation {
    Write(PathBuf, Cow<'static, [u8]>),
    Delete(PathBuf),
}

impl FilesystemOperation {
    /// Executes the filesystem operation.
    pub fn execute(self) -> Result<(), DatabaseError> {
        match self {
            Self::Write(path, content) => {
                std::fs::write(path, content)?;

                Ok(())
            }
            Self::Delete(path) => {
                std::fs::remove_file(path)?;

                Ok(())
            }
        }
    }
}
