//! Database watcher for real-time file change monitoring.

use std::borrow::Cow;
use std::collections::HashSet;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use notify::Config;
use notify::Event;
use notify::RecommendedWatcher;
use notify::RecursiveMode;
use notify::Watcher as NotifyWatcher;

use crate::Database;
use crate::DatabaseReader;
use crate::ReadDatabase;
use crate::error::DatabaseError;
use crate::exclusion::Exclusion;
use crate::file::File;
use crate::file::FileId;
use crate::file::FileType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ChangedFile {
    id: FileId,
    path: PathBuf,
}

/// Options for configuring the file system watcher.
#[derive(Debug, Clone)]
pub struct WatchOptions {
    pub poll_interval: Option<Duration>,
    pub additional_excludes: Vec<Exclusion<'static>>,
}

impl Default for WatchOptions {
    fn default() -> Self {
        Self { poll_interval: Some(Duration::from_millis(500)), additional_excludes: vec![] }
    }
}

/// Database watcher service that monitors file changes and updates the database.
pub struct DatabaseWatcher<'a> {
    database: Database<'a>,
    watcher: Option<RecommendedWatcher>,
    watched_paths: Vec<PathBuf>,
    receiver: Option<Receiver<Vec<ChangedFile>>>,
}

impl<'a> DatabaseWatcher<'a> {
    pub fn new(database: Database<'a>) -> Self {
        Self { database, watcher: None, watched_paths: Vec::new(), receiver: None }
    }

    pub fn watch(&mut self, options: WatchOptions) -> Result<(), DatabaseError> {
        self.stop();

        let config = &self.database.configuration;

        let (tx, rx) = mpsc::channel();

        let mut all_exclusions = vec![
            Exclusion::Pattern(Cow::Borrowed("**/node_modules/**")),
            Exclusion::Pattern(Cow::Borrowed("**/.git/**")),
            Exclusion::Pattern(Cow::Borrowed("**/.idea/**")),
            Exclusion::Pattern(Cow::Borrowed("**/vendor/**")),
        ];
        all_exclusions.extend(config.excludes.iter().cloned());
        all_exclusions.extend(options.additional_excludes);

        let mut glob_builder = GlobSetBuilder::new();
        for ex in &all_exclusions {
            if let Exclusion::Pattern(pat) = ex {
                glob_builder.add(Glob::new(pat)?);
            }
        }
        let glob_excludes = glob_builder.build()?;

        let path_excludes: HashSet<PathBuf> = all_exclusions
            .iter()
            .filter_map(|ex| match ex {
                Exclusion::Path(p) => Some(p.as_ref().to_path_buf()),
                _ => None,
            })
            .collect();

        let extensions: HashSet<String> = config.extensions.iter().map(|s| s.to_string()).collect();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res
                    && let Some(changed) = Self::handle_event(event, &glob_excludes, &path_excludes, &extensions)
                {
                    let _ = tx.send(changed);
                }
            },
            Config::default().with_poll_interval(options.poll_interval.unwrap_or(Duration::from_millis(500))),
        )
        .map_err(DatabaseError::WatcherInit)?;

        let mut watched_paths = Vec::new();
        for path in &config.paths {
            let path_buf = Path::new(path.as_ref()).to_path_buf();
            watcher.watch(&path_buf, RecursiveMode::Recursive).map_err(DatabaseError::WatcherWatch)?;
            watched_paths.push(path_buf.clone());
            tracing::debug!("Watching path: {}", path_buf.display());
        }
        for path in &config.includes {
            let path_buf = Path::new(path.as_ref()).to_path_buf();
            watcher.watch(&path_buf, RecursiveMode::Recursive).map_err(DatabaseError::WatcherWatch)?;
            watched_paths.push(path_buf.clone());
            tracing::debug!("Watching include path: {}", path_buf.display());
        }

        tracing::info!("Database watcher started for workspace: {}", config.workspace.display());

        self.watcher = Some(watcher);
        self.watched_paths = watched_paths;
        self.receiver = Some(rx);

        Ok(())
    }

    /// Stops watching if currently active.
    pub fn stop(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            for path in &self.watched_paths {
                let _ = watcher.unwatch(path);
                tracing::debug!("Stopped watching: {}", path.display());
            }
        }
        self.watched_paths.clear();
        self.receiver = None;
    }

    /// Checks if the watcher is currently active.
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }

    /// Handles a file system event and returns changed files with their paths.
    fn handle_event(
        event: Event,
        glob_excludes: &GlobSet,
        path_excludes: &HashSet<PathBuf>,
        extensions: &HashSet<String>,
    ) -> Option<Vec<ChangedFile>> {
        tracing::debug!("Watcher received event: kind={:?}, paths={:?}", event.kind, event.paths);

        let mut changed_files = Vec::new();

        for path in event.paths {
            // Check if file has a valid extension
            if let Some(ext) = path.extension() {
                if !extensions.contains(ext.to_string_lossy().as_ref()) {
                    continue;
                }
            } else {
                continue;
            }

            // Check glob pattern exclusions
            if glob_excludes.is_match(&path) {
                tracing::debug!("Skipping path excluded by pattern: {}", path.display());
                continue;
            }

            // Check exact path exclusions
            if path_excludes.contains(&path) {
                tracing::debug!("Skipping excluded path: {}", path.display());
                continue;
            }

            // Check if any parent directory is in path_excludes
            let mut should_skip = false;
            for ancestor in path.ancestors().skip(1) {
                if path_excludes.contains(ancestor) {
                    tracing::debug!("Skipping path under excluded directory: {}", path.display());
                    should_skip = true;
                    break;
                }
            }
            if should_skip {
                continue;
            }

            // Create file ID from path
            let file_name = path.to_string_lossy();
            let file_id = FileId::new(file_name.as_ref());

            changed_files.push(ChangedFile { id: file_id, path: path.clone() });
        }

        if changed_files.is_empty() { None } else { Some(changed_files) }
    }

    /// Waits for file changes and updates the database.
    ///
    /// This method blocks until file changes are detected, then updates the database
    /// in place and returns the IDs of changed files.
    ///
    /// # Returns
    ///
    /// - `Ok(file_ids)` - The IDs of files that were changed (empty if no changes)
    /// - `Err(DatabaseError::WatcherNotActive)` - If the watcher is not currently watching
    /// - `Err(e)` - If updating the database failed
    pub fn wait(&mut self) -> Result<Vec<FileId>, DatabaseError> {
        let Some(receiver) = &self.receiver else {
            return Err(DatabaseError::WatcherNotActive);
        };

        let config = &self.database.configuration;
        let workspace = config.workspace.as_ref().to_path_buf();

        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(changed_files) => {
                std::thread::sleep(Duration::from_millis(250));
                let mut all_changed = changed_files;
                while let Ok(more) = receiver.try_recv() {
                    all_changed.extend(more);
                }

                all_changed.sort_by_key(|f| f.id);
                all_changed.dedup_by_key(|f| f.id);

                let mut changed_ids = Vec::new();

                for changed_file in &all_changed {
                    changed_ids.push(changed_file.id);

                    match self.database.get(&changed_file.id) {
                        Ok(file) => {
                            if changed_file.path.exists() {
                                match std::fs::read_to_string(&changed_file.path) {
                                    Ok(contents) => {
                                        self.database.update(changed_file.id, Cow::Owned(contents));
                                        tracing::debug!("Updated file in database: {}", file.name);
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to read file {}: {}", changed_file.path.display(), e);
                                    }
                                }
                            } else {
                                self.database.delete(changed_file.id);
                                tracing::debug!("Deleted file from database: {}", file.name);
                            }
                        }
                        Err(_) => {
                            if changed_file.path.exists() {
                                match File::read(&workspace, &changed_file.path, FileType::Host) {
                                    Ok(file) => {
                                        self.database.add(file);
                                        tracing::debug!("Added new file to database: {}", changed_file.path.display());
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Failed to load new file {}: {}",
                                            changed_file.path.display(),
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(changed_ids)
            }
            Err(RecvTimeoutError::Timeout) => Ok(Vec::new()),
            Err(RecvTimeoutError::Disconnected) => {
                self.stop();
                Err(DatabaseError::WatcherNotActive)
            }
        }
    }

    /// Returns a reference to the database.
    pub fn database(&self) -> &Database<'a> {
        &self.database
    }

    /// Returns a reference to the database.
    pub fn read_only_database(&self) -> ReadDatabase {
        self.database.read_only()
    }

    /// Returns a mutable reference to the database.
    pub fn database_mut(&mut self) -> &mut Database<'a> {
        &mut self.database
    }

    /// Provides temporary mutable access to the database through a closure.
    ///
    /// This method helps Rust's borrow checker understand that the mutable borrow
    /// of the database is scoped to just the closure execution, allowing the watcher
    /// to be used again after the closure returns.
    ///
    /// The closure is bounded with for<'x> to explicitly show that the database
    /// reference lifetime is scoped to the closure execution only.
    pub fn with_database_mut<F, R>(&mut self, f: F) -> R
    where
        F: for<'x> FnOnce(&'x mut Database<'a>) -> R,
    {
        f(&mut self.database)
    }

    /// Consumes the watcher and returns the database.
    pub fn into_database(self) -> Database<'a> {
        let mut md = ManuallyDrop::new(self);
        md.stop();
        unsafe { std::ptr::read(&md.database) }
    }
}

impl<'a> Drop for DatabaseWatcher<'a> {
    fn drop(&mut self) {
        self.stop();
    }
}
