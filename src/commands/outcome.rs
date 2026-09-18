use std::collections::BTreeMap;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::process::ExitCode;

use mago_database::DatabaseReader;
use mago_database::file::FileId;

use crate::error::Error;

#[derive(Debug)]
pub struct CommandOutcome {
    pub exit_code: ExitCode,
    pub changed_files: BTreeMap<FileId, u64>,
}

impl CommandOutcome {
    pub fn with_changes(
        exit_code: ExitCode,
        database: &impl DatabaseReader,
        file_ids: impl IntoIterator<Item = FileId>,
    ) -> Result<Self, Error> {
        let mut outcome = Self::from(exit_code);
        for file_id in file_ids {
            let file = database.get_ref(&file_id)?;
            let mut hasher = DefaultHasher::new();
            file.contents.hash(&mut hasher);
            outcome.changed_files.insert(file_id, hasher.finish());
        }

        Ok(outcome)
    }
}

impl From<ExitCode> for CommandOutcome {
    fn from(exit_code: ExitCode) -> Self {
        Self { exit_code, changed_files: BTreeMap::new() }
    }
}
