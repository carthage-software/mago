#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings as AnalyzerSettings;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_database::DatabaseConfiguration;
use mago_database::DatabaseReader;
use mago_database::GlobSettings;
use mago_database::file::{File, FileType};
use mago_database::loader::DatabaseLoader;
use mago_database::membership::WorkspaceMatcher;
use mago_formatter::settings::FormatSettings;
use mago_linter::settings::Settings as LinterSettings;
use mago_php_version::PHPVersion;
use mago_reporting::CompiledIgnoreSet;
use mago_syntax::settings::ParserSettings;

use super::*;
use crate::workspace::directory_aliases::DirectoryAliases;
use crate::{Features, Server, Settings};

fn workspace(root: &Path) -> Workspace<u64> {
    let root = root.canonicalize().unwrap();
    workspace_with(&root, DatabaseConfiguration::new(&root, vec![b"."], vec![], vec![], vec![b"php"]))
}

fn workspace_with(root: &Path, configuration: DatabaseConfiguration<'_>) -> Workspace<u64> {
    let root = root.to_path_buf();
    let matcher = WorkspaceMatcher::from_configuration(&configuration).unwrap();
    let (database, links) = DatabaseLoader::new(configuration.into_static()).load_with_directory_symlinks().unwrap();
    let mut server = Server::new(
        database,
        CodebaseMetadata::new(),
        SymbolReferences::new(),
        Settings {
            php_version: PHPVersion::default(),
            features: Features { analyzer: true, linter: false, formatter: true },
            parser: ParserSettings::default(),
            analyzer: AnalyzerSettings::default(),
            linter: LinterSettings::default(),
            formatter: FormatSettings::default(),
            plugin_registry: Arc::new(PluginRegistry::new()),
            excludes: crate::Excludes::default(),
        },
    );
    server.analyze_without_collecting().unwrap();
    let mut workspace =
        Workspace::new(root.clone(), root, server, matcher, CompiledIgnoreSet::compile(&[], GlobSettings::default()));
    workspace.directory_aliases = DirectoryAliases::new(links, &workspace.matcher);
    workspace
}

fn file(workspaces: &Workspaces<u64>, path: &Path) -> Arc<File> {
    let index = workspaces.owner_index(path).unwrap();
    DocumentPath::new(path).file(&workspaces.entries()[index]).unwrap()
}

fn event(path: &Path, kind: FileChange) -> Vec<FileEvent> {
    vec![FileEvent { path: path.to_path_buf(), kind }]
}

#[test]
fn buffer_versions_and_disk_changes_keep_accepted_contents() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("source.php");
    std::fs::write(&path, "<?php class Original {}").unwrap();
    let mut workspaces = Workspaces::new(vec![workspace(root.path())]);
    assert_eq!(workspaces.open_buffer(42, path.clone(), b"<?php class Edited {}".to_vec(), 7).changed_files, 1);
    for version in [6, 7] {
        assert!(!workspaces.change_buffer(42, b"<?php class Stale {}".to_vec(), version).queries_changed);
    }
    assert!(!workspaces.change_buffer(43, b"<?php class Other {}".to_vec(), 8).queries_changed);
    assert_eq!(workspaces.document_version(&42), Some(7));
    assert_eq!(workspaces.change_buffer(42, b"<?php class Latest {}".to_vec(), 8).changed_files, 1);
    assert_eq!(workspaces.entries()[0].server.get_symbols("Latest").len(), 1);

    std::fs::write(&path, "<?php class Disk {}").unwrap();
    assert_eq!(workspaces.change_disk_files(event(&path, FileChange::Changed)).changed_files, 0);
    assert_eq!(file(&workspaces, &path).contents, b"<?php class Latest {}".as_slice());
    assert_eq!(workspaces.close_buffer(&42).changed_files, 1);
    assert_eq!(workspaces.document_version(&42), None);
    assert!(workspaces.entries()[0].server.get_symbols("Latest").is_empty());
    assert_eq!(workspaces.entries()[0].server.get_symbols("Disk").len(), 1);
}

#[test]
fn buffers_survive_root_changes_and_share_a_live_database_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let child = root.join("child");
    std::fs::create_dir_all(&child).unwrap();
    let path = child.join("source.php");
    std::fs::write(&path, "<?php class Original {}").unwrap();
    let mut workspaces = Workspaces::new(vec![workspace(&root)]);
    workspaces.open_buffer(1, path.clone(), b"<?php class Edited {}".to_vec(), 1);
    let previous = Arc::downgrade(&file(&workspaces, &path));
    assert_eq!(workspaces.add(workspace(&child)).unwrap(), 1);
    assert_eq!(workspaces.entries()[0].root, child);
    assert!(Arc::ptr_eq(&workspaces.documents()[&1].snapshot, &file(&workspaces, &path)));
    assert!(workspaces.remove(&root));
    assert!(previous.upgrade().is_none());
    assert!(workspaces.remove(&child));
    workspaces.change_buffer(1, b"<?php class Latest {}".to_vec(), 2);
    assert_eq!(workspaces.add(workspace(&root)).unwrap(), 1);
    assert_eq!(file(&workspaces, &path).contents, b"<?php class Latest {}".as_slice());
    assert_eq!(workspaces.entries()[0].server.get_symbols("Latest").len(), 1);
}

#[test]
fn replaced_workspaces_reload_disk_and_keep_accepted_buffers() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let path = root.join("source.php");
    std::fs::write(&path, "<?php class Disk {}").unwrap();
    let mut workspaces = Workspaces::new(vec![workspace(&root)]);
    workspaces.open_buffer(1, path.clone(), b"<?php class Buffer {}".to_vec(), 1);
    std::fs::write(root.join("added.php"), "<?php class Added {}").unwrap();
    assert_eq!(workspaces.replace(workspace(&root)).unwrap(), Some(1));
    let server = &workspaces.entries()[0].server;
    assert_eq!(server.get_symbols("Buffer").len(), 1);
    assert_eq!(server.get_symbols("Added").len(), 1);
    assert!(server.get_symbols("Disk").is_empty());
    assert!(Arc::ptr_eq(&workspaces.documents()[&1].snapshot, &file(&workspaces, &path)));

    let elsewhere = tempfile::tempdir().unwrap();
    assert!(workspaces.replace(workspace(elsewhere.path())).unwrap().is_none());
    assert_eq!(workspaces.entries().len(), 1);
}

#[test]
fn directory_deletion_keeps_buffers_and_releases_deleted_analysis() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let package = root.join("package");
    std::fs::create_dir_all(&package).unwrap();
    let keep = package.join("Keep.php");
    let gone = package.join("Gone.php");
    for path in [&keep, &gone] {
        std::fs::write(path, "<?php").unwrap();
    }
    let mut workspaces = Workspaces::new(vec![workspace(&root)]);
    let id = file(&workspaces, &gone).id;
    let analysis = {
        let analysis = workspaces.entries_mut()[0].server.file_analysis_for(id).unwrap();
        Arc::downgrade(&analysis)
    };
    workspaces.open_buffer(1, keep.clone(), b"<?php class Buffer {}".to_vec(), 1);
    std::fs::remove_dir_all(&package).unwrap();
    assert_eq!(workspaces.change_disk_files(event(&package, FileChange::Deleted)).changed_files, 1);
    assert!(analysis.upgrade().is_none());
    assert_eq!(file(&workspaces, &keep).contents, b"<?php class Buffer {}".as_slice());
    assert_eq!(workspaces.close_buffer(&1).changed_files, 1);
    assert!(workspaces.entries()[0].database().files().next().is_none());
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(&gone, "<?php class Restored {}").unwrap();
    assert_eq!(workspaces.change_disk_files(event(&package, FileChange::Created)).changed_files, 1);
    assert_eq!(file(&workspaces, &gone).id, id);
    assert_eq!(workspaces.entries()[0].server.get_symbols("Restored").len(), 1);
}

#[test]
fn created_directories_below_a_source_root_follow_the_loaders_pruning() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join("generated")).unwrap();
    std::fs::write(root.join("generated/Top.php"), "<?php class Top {}").unwrap();
    let mut configuration = DatabaseConfiguration::new(
        &root,
        vec![b"generated"],
        vec![],
        vec![mago_database::exclusion::Exclusion::Pattern("generated/*".into())],
        vec![b"php"],
    );
    configuration.glob.literal_separator = true;
    let mut workspaces = Workspaces::new(vec![workspace_with(&root, configuration)]);
    assert!(workspaces.entries()[0].database().files().next().is_none());

    let deep = root.join("generated/deep");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("New.php"), "<?php class Created {}").unwrap();
    assert_eq!(workspaces.change_disk_files(event(&deep, FileChange::Created)).changed_files, 1);
    assert_eq!(workspaces.entries()[0].server.get_symbols("Created").len(), 1);

    std::fs::remove_dir_all(&deep).unwrap();
    assert_eq!(workspaces.change_disk_files(event(&deep, FileChange::Deleted)).changed_files, 1);
    assert!(workspaces.entries()[0].database().files().next().is_none());

    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("New.php"), "<?php class Created {}").unwrap();
    assert_eq!(workspaces.change_disk_files(event(&deep, FileChange::Created)).changed_files, 1);
    std::fs::remove_dir_all(&deep).unwrap();
    let mut batch = event(&deep, FileChange::Deleted);
    batch.extend(event(&deep.join("New.php"), FileChange::Deleted));
    assert_eq!(workspaces.change_disk_files(batch).changed_files, 1);
    assert!(workspaces.entries()[0].database().files().next().is_none());

    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("New.php"), "<?php class Created {}").unwrap();
    assert_eq!(workspaces.change_disk_files(event(&deep, FileChange::Created)).changed_files, 1);
    std::fs::remove_dir_all(&deep).unwrap();
    let mut batch: Vec<_> = (0..64)
        .map(|index| FileEvent { path: root.join(format!("generated/Excluded{index}.php")), kind: FileChange::Deleted })
        .collect();
    batch.extend(event(&deep, FileChange::Deleted));
    assert_eq!(workspaces.change_disk_files(batch).changed_files, 1);
    assert!(workspaces.entries()[0].database().files().next().is_none());
}

#[test]
fn deletions_at_mixed_depths_release_only_files_below_them() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let deep = root.join("shallow/a/b/Deep.php");
    let near = root.join("nested/c/d/Near.php");
    let keep = root.join("nested/Keep.php");
    for path in [&deep, &near, &keep] {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "<?php").unwrap();
    }
    let mut workspaces = Workspaces::new(vec![workspace(&root)]);
    std::fs::remove_dir_all(root.join("shallow")).unwrap();
    std::fs::remove_dir_all(root.join("nested/c")).unwrap();
    let batch = [root.join("shallow"), root.join("nested/c/d"), root.join("x/y/z/w/v/Unknown.tmp")]
        .into_iter()
        .map(|path| FileEvent { path, kind: FileChange::Deleted })
        .collect();
    assert_eq!(workspaces.change_disk_files(batch).changed_files, 2);
    let paths: Vec<_> = workspaces.entries()[0]
        .database()
        .files()
        .filter_map(|file| file.path.as_deref().map(Path::to_path_buf))
        .collect();
    assert_eq!(paths, [keep]);
}

#[cfg(unix)]
#[test]
fn deleted_package_directories_reach_their_glob_matched_aliases() {
    let directory = tempfile::tempdir().unwrap();
    let parent = directory.path().canonicalize().unwrap();
    let target = parent.join("package");
    let root = parent.join("workspace/project");
    std::fs::create_dir_all(target.join("sub")).unwrap();
    std::fs::create_dir_all(root.join("vendor")).unwrap();
    std::fs::write(target.join("sub/Contact.php"), "<?php class Contact {}").unwrap();
    let link = root.join("vendor/package");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let mut configuration = DatabaseConfiguration::new(&root, vec![b"src"], vec![b"vendor"], vec![], vec![b"php"]);
    configuration.excludes.push(mago_database::exclusion::Exclusion::Pattern("**/sub".into()));
    let mut workspaces = Workspaces::new(vec![workspace_with(&root, configuration)]);
    let loaded = link.join("sub/Contact.php");
    assert!(workspaces.entries()[0].database().files().any(|file| file.path.as_deref() == Some(loaded.as_path())));

    std::fs::remove_dir_all(target.join("sub")).unwrap();
    assert_eq!(DocumentPath::new(&link.join("sub")).resolved(), target.join("sub"));
    assert_eq!(workspaces.change_disk_files(event(&target.join("sub"), FileChange::Deleted)).changed_files, 1);
    assert!(workspaces.entries()[0].database().files().next().is_none());
}

#[cfg(unix)]
#[test]
fn a_second_buffer_key_cannot_replace_an_accepted_path_alias() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("source.php");
    let alias = root.path().join("alias.php");
    std::fs::write(&path, "<?php").unwrap();
    std::os::unix::fs::symlink(&path, &alias).unwrap();
    let mut workspaces = Workspaces::new(vec![workspace(root.path())]);
    workspaces.open_buffer(1, path.clone(), b"<?php class First {}".to_vec(), 1);
    assert!(!workspaces.open_buffer(2, alias, b"<?php class Second {}".to_vec(), 1).queries_changed);
    assert_eq!(workspaces.document_version(&2), None);
    std::fs::remove_file(&path).unwrap();
    workspaces.change_buffer(1, b"<?php class StillOpen {}".to_vec(), 2);
    assert_eq!(file(&workspaces, &path).contents, b"<?php class StillOpen {}".as_slice());
    assert_eq!(workspaces.close_buffer(&1).changed_files, 1);
}

#[test]
fn published_builtin_paths_reject_buffer_edits() {
    let root = tempfile::tempdir().unwrap();
    let sources = tempfile::tempdir().unwrap();
    let mut workspace = workspace(root.path());
    let builtin = workspace.database_mut().add(File::new(
        b"Core/strlen.php".as_slice().into(),
        FileType::Builtin,
        None,
        b"<?php".as_slice().into(),
    ));
    workspace.set_source_root(sources.path().to_path_buf());
    let path = sources.path().join("Core/strlen.php");
    let mut workspaces = Workspaces::new(vec![workspace]);
    assert_eq!(file(&workspaces, &path).id, builtin);
    assert!(!workspaces.open_buffer(1, path, b"changed".to_vec(), 1).queries_changed);
    assert!(workspaces.documents().is_empty());
    assert_eq!(workspaces.entries()[0].database().get(&builtin).unwrap().contents, b"<?php".as_slice());
}
