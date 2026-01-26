//! Database watcher for real-time file change monitoring.

use std::borrow::Cow;
use std::collections::HashMap;
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
use notify::EventKind;
use notify::RecommendedWatcher;
use notify::RecursiveMode;
use notify::Watcher as NotifyWatcher;
use notify::event::ModifyKind;

use crate::Database;
use crate::DatabaseReader;
use crate::ReadDatabase;
use crate::error::DatabaseError;
use crate::exclusion::Exclusion;
use crate::file::File;
use crate::file::FileId;
use crate::file::FileType;

const DEFAULT_POLL_INTERVAL_MS: u64 = 1000;

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
        Self { poll_interval: Some(Duration::from_millis(DEFAULT_POLL_INTERVAL_MS)), additional_excludes: vec![] }
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
    #[must_use]
    pub fn new(database: Database<'a>) -> Self {
        Self { database, watcher: None, watched_paths: Vec::new(), receiver: None }
    }

    /// Starts watching for file changes in the configured directories.
    ///
    /// # Errors
    ///
    /// Returns a [`DatabaseError`] if:
    /// - A glob pattern is invalid
    /// - The file system watcher cannot be created
    /// - Directories cannot be watched
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

        let extensions: HashSet<String> = config.extensions.iter().map(std::string::ToString::to_string).collect();
        let workspace = config.workspace.as_ref().to_path_buf();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res
                    && let Some(changed) =
                        Self::handle_event(event, &workspace, &glob_excludes, &path_excludes, &extensions)
                {
                    let _ = tx.send(changed);
                }
            },
            Config::default()
                .with_poll_interval(options.poll_interval.unwrap_or(Duration::from_millis(DEFAULT_POLL_INTERVAL_MS))),
        )
        .map_err(DatabaseError::WatcherInit)?;

        let mut unique_watch_paths = HashSet::new();

        for path in &config.paths {
            let watch_path = Self::extract_watch_path(path.as_ref());
            let absolute_path = if watch_path.is_absolute() { watch_path } else { config.workspace.join(watch_path) };

            unique_watch_paths.insert(absolute_path);
        }

        for path in &config.includes {
            let watch_path = Self::extract_watch_path(path.as_ref());
            let absolute_path = if watch_path.is_absolute() { watch_path } else { config.workspace.join(watch_path) };

            unique_watch_paths.insert(absolute_path);
        }

        let mut watched_paths = Vec::new();
        for path in unique_watch_paths {
            watcher.watch(&path, RecursiveMode::Recursive).map_err(DatabaseError::WatcherWatch)?;
            watched_paths.push(path.clone());
            tracing::debug!("Watching path: {}", path.display());
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
    #[must_use]
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }

    /// Extracts the base directory path from a potentially glob-pattern path.
    ///
    /// For glob patterns (containing *, ?, [, {), this returns the directory portion
    /// before the first glob metacharacter. For regular paths, returns the path as-is.
    ///
    /// # Examples
    ///
    /// - `"src/**/*.php"` → `"src"`
    /// - `"lib/*/foo.php"` → `"lib"`
    /// - `"tests/fixtures"` → `"tests/fixtures"` (unchanged)
    fn extract_watch_path(pattern: &str) -> PathBuf {
        let is_glob = pattern.contains('*') || pattern.contains('?') || pattern.contains('[') || pattern.contains('{');

        if !is_glob {
            return PathBuf::from(pattern);
        }

        let first_glob_pos = pattern.find(['*', '?', '[', '{']).unwrap_or(pattern.len());

        let base = &pattern[..first_glob_pos];

        let base = base.trim_end_matches('/').trim_end_matches('\\');

        if base.is_empty() { PathBuf::from(".") } else { PathBuf::from(base) }
    }

    fn handle_event(
        event: Event,
        workspace: &Path,
        glob_excludes: &GlobSet,
        path_excludes: &HashSet<PathBuf>,
        extensions: &HashSet<String>,
    ) -> Option<Vec<ChangedFile>> {
        tracing::debug!("Watcher received event: kind={:?}, paths={:?}", event.kind, event.paths);

        if let EventKind::Other | EventKind::Any | EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_)) =
            event.kind
        {
            tracing::debug!("Ignoring non-modification event: {:?}", event.kind);

            return None;
        }

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

            // Normalize to forward slashes for cross-platform determinism
            let logical_name = path.strip_prefix(workspace).unwrap_or(&path).to_string_lossy().replace('\\', "/");
            let file_id = FileId::new(&logical_name);

            changed_files.push(ChangedFile { id: file_id, path: path.clone() });
        }

        if changed_files.is_empty() { None } else { Some(changed_files) }
    }

    /// Waits for file changes and updates the database.
    ///
    /// This method blocks until file changes are detected, then updates the database
    /// in place and returns the IDs of changed files.
    ///
    /// # Errors
    ///
    /// Returns a [`DatabaseError`] if:
    /// - The watcher is not currently active ([`DatabaseError::WatcherNotActive`])
    /// - Updating the database with changed files fails
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

                let mut latest_changes: HashMap<FileId, ChangedFile> = HashMap::new();
                for changed in all_changed {
                    latest_changes.insert(changed.id, changed);
                }
                let all_changed: Vec<ChangedFile> = latest_changes.into_values().collect();
                let mut changed_ids = Vec::new();

                for changed_file in &all_changed {
                    changed_ids.push(changed_file.id);

                    match self.database.get(&changed_file.id) {
                        Ok(file) => {
                            if changed_file.path.exists() {
                                match std::fs::read_to_string(&changed_file.path) {
                                    Ok(contents) => {
                                        self.database.update(changed_file.id, Cow::Owned(contents));
                                        tracing::trace!("Updated file in database: {}", file.name);
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to read file {}: {}", changed_file.path.display(), e);
                                    }
                                }
                            } else {
                                self.database.delete(changed_file.id);
                                tracing::trace!("Deleted file from database: {}", file.name);
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
    #[must_use]
    pub fn database(&self) -> &Database<'a> {
        &self.database
    }

    /// Returns a reference to the database.
    #[must_use]
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
    #[must_use]
    pub fn into_database(self) -> Database<'a> {
        let mut md = ManuallyDrop::new(self);
        md.stop();
        unsafe { std::ptr::read(&raw const md.database) }
    }
}

impl Drop for DatabaseWatcher<'_> {
    fn drop(&mut self) {
        self.stop();
    }
}
